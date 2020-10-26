use crate::broker::Broker;
use crate::error::RunError;
use crate::map::Map;
use crate::processor::{FnContainer, Processor};
use ctx_thread::scope;
use futures::{stream::select, Sink, SinkExt, Stream, StreamExt};
use scipio::LocalExecutorBuilder;
use std::error::Error;
use std::sync::Arc;

pub struct Reactor<S, P, B> {
    source: S,
    processor: P,
    broker: B,
}

impl<
        'a,
        O,
        E: Error,
        S: Stream<Item = Result<O, E>> + Unpin + Sync + Send,
        P: Processor<'a, Input = O> + Send + Sync,
        B: Broker<O> + Send + Sync,
    > Reactor<S, P, B>
{
    pub fn run<SINK: Sink<P::Output> + Unpin + Send + Sync>(
        self,
        sink: &mut SINK,
    ) -> Result<(), RunError<E>> {
        let broker = Arc::new(self.broker);
        let processor = self.processor;
        let mut source = self.source;

        scope(|ctx| {
            let inc = broker.clone();

            ctx.spawn(|ctx| {
                let executor = LocalExecutorBuilder::new().make().unwrap();
                executor.run(async move {
                    while let Some(Ok(item)) = source.next().await {
                        if ctx.active() {
                            broker.submit(item);
                        }
                    }
                    ctx.cancel();
                });
            });

            let executor = LocalExecutorBuilder::new().make().unwrap();
            executor.run(async move {
                while ctx.active() {
                    if let Some(val) = inc.claim() {
                        let result = processor.process(val);
                        if result.is_err() {
                            sink.flush().await;
                        }
                        sink.send(result.unwrap()).await;
                    }
                }
            });
        });
        Ok(())
    }
}

impl Reactor<(), (), ()> {
    pub fn builder() -> ReactorBuilder<(), (), ()> {
        ReactorBuilder::<(), (), ()>::new()
    }
}

impl<
        'a,
        O,
        E,
        SOURCE: Stream<Item = Result<O, E>> + Unpin + Sync + Send,
        PROCESSOR: Processor<'a, Input = O>,
        BROKER: Broker<O>,
    > Reactor<SOURCE, PROCESSOR, BROKER>
{
    pub fn new(
        source: SOURCE,
        processor: PROCESSOR,
        broker: BROKER,
    ) -> Reactor<SOURCE, PROCESSOR, BROKER> {
        Reactor {
            source,
            processor,
            broker,
        }
    }
}

pub struct ReactorBuilder<SO, P, B> {
    source: SO,
    processor: P,
    broker: B,
}

impl ReactorBuilder<(), (), ()> {
    pub fn new() -> ReactorBuilder<(), (), ()> {
        ReactorBuilder {
            source: (),
            processor: (),
            broker: (),
        }
    }
}

impl<SO, P, B> ReactorBuilder<SO, P, B>
where
    SO: Stream,
{
    // TODO: to use this, the syntax requires .combine::<_, O>(...) (the S2 argument can be elided,
    // but perhaps cleverly using impl Source somewhere in the definition will allow us to write
    // .combine::<O>(...)
    pub fn combine_into<S2: Stream, O>(self, source: S2) -> ReactorBuilder<impl Stream, P, B>
    where
        O: From<SO::Item> + From<S2::Item>,
    {
        ReactorBuilder {
            source: select(
                self.source.map(Into::<O>::into),
                source.map(Into::<O>::into),
            ),
            broker: self.broker,
            processor: self.processor,
        }
    }
}

impl<
        'a,
        O,
        E,
        SOURCE: Stream<Item = Result<O, E>> + Unpin + Sync + Send,
        PROCESSOR: Processor<'a, Input = O>,
        BROKER: Broker<O>,
    > ReactorBuilder<SOURCE, PROCESSOR, BROKER>
{
    pub fn build(self) -> Reactor<SOURCE, PROCESSOR, BROKER> {
        Reactor::new(self.source, self.processor, self.broker)
    }
}

impl<'a, S, P, B> ReactorBuilder<S, P, B>
where
    P: Processor<'a>,
{
    pub fn pipe<F: Processor<'a, Input = P::Output>>(
        self,
        processor: F,
    ) -> ReactorBuilder<S, Map<P, F>, B> {
        ReactorBuilder {
            processor: self.processor.pipe(processor),
            source: self.source,
            broker: self.broker,
        }
    }

    pub fn map<F: Fn(P::Output) -> O, O>(
        self,
        f: F,
    ) -> ReactorBuilder<S, Map<P, FnContainer<F, P::Output, O>>, B> {
        ReactorBuilder {
            processor: self.processor.map(f),
            source: self.source,
            broker: self.broker,
        }
    }
}

impl<'a, O, E, SOURCE: Stream<Item = Result<O, E>> + Unpin + Sync + Send, B>
    ReactorBuilder<SOURCE, (), B>
{
    pub fn pipe<F: Processor<'a, Input = O>>(self, processor: F) -> ReactorBuilder<SOURCE, F, B> {
        ReactorBuilder {
            processor,
            source: self.source,
            broker: self.broker,
        }
    }

    pub fn map<F: Fn(O) -> OUT, OUT>(
        self,
        f: F,
    ) -> ReactorBuilder<SOURCE, FnContainer<F, O, OUT>, B> {
        ReactorBuilder {
            processor: FnContainer::new(f),
            source: self.source,
            broker: self.broker,
        }
    }
}

impl<'a, S, P, B> ReactorBuilder<S, P, B> {
    pub fn with_broker<B2>(self, broker: B2) -> ReactorBuilder<S, P, B2> {
        ReactorBuilder {
            broker,
            source: self.source,
            processor: self.processor,
        }
    }

    pub fn with_source<S2>(self, source: S2) -> ReactorBuilder<S2, P, B> {
        ReactorBuilder {
            source,
            processor: self.processor,
            broker: self.broker,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Er;
    use crossbeam_queue::ArrayQueue;
    use futures::stream::iter;

    #[test]
    fn test_single_producer_single_consumer_run() {
        let mut result = Vec::new();
        let reactor = Reactor::builder()
            .with_source(iter(vec![Ok(1), Err(Er::E)]))
            .with_broker(ArrayQueue::new(2))
            .map(|i| i + 1)
            .build();
        reactor.run(&mut result).unwrap();
        assert_eq!(result, vec![2])
    }
}
