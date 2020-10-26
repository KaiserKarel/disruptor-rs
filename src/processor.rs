use crate::map::Map;
use std::error::Error;
use std::marker::PhantomData;

pub trait Processor<'state> {
    type Input;
    type Output;
    type Error: Error + Send;
    type State;

    fn state(&self) -> Self::State;
    fn set_state(&mut self, state: Self::State);

    fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    fn pipe<F: Processor<'state>>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map {
            inner: self,
            outer: f,
        }
    }

    fn map<F: Fn(Self::Output) -> O, O>(self, f: F) -> Map<Self, FnContainer<F, Self::Output, O>>
    where
        Self: Sized,
    {
        Map {
            inner: self,
            outer: FnContainer {
                f,
                i: Default::default(),
                o: Default::default(),
            },
        }
    }
}

pub struct FnContainer<F, I, O> {
    f: F,
    i: PhantomData<I>,
    o: PhantomData<O>,
}

impl<F, I, O> FnContainer<F, I, O> {
    pub fn new(f: F) -> FnContainer<F, I, O> {
        FnContainer {
            f,
            i: Default::default(),
            o: Default::default(),
        }
    }
}

impl<'state, F: Fn(I) -> O, I, O> Processor<'state> for FnContainer<F, I, O> {
    type Input = I;
    type Output = O;
    type Error = !;
    type State = ();

    fn state(&self) -> Self::State {}

    fn set_state(&mut self, _: Self::State) {}

    fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self.f.call((input,)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        // let processor = |()| { Ok(()) };
        // let processor = processor.map(|r| {
        //     println!("received message");
        //     r
        // });
    }
}
