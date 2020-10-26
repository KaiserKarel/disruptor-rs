use crate::processor::Processor;

// Maps A -> B, B -> C to A -> C, using appropriate From implementations where needed.
pub struct Map<P, F> {
    pub(crate) inner: P,
    pub(crate) outer: F,
}

impl<'a, A, B, C, D: From<B>, INNER, OUTER> Processor<'a> for Map<INNER, OUTER>
where
    INNER: Processor<'a, Input = A, Output = B> + 'a,
    OUTER: Processor<'a, Input = D, Output = C> + 'a,
    OUTER::Error: From<INNER::Error>,
{
    type Input = INNER::Input;
    type Output = OUTER::Output;
    type Error = OUTER::Error;
    type State = (INNER::State, OUTER::State);

    fn state(&self) -> Self::State {
        (self.inner.state(), self.outer.state())
    }

    fn set_state(&mut self, (inner, outer): Self::State) {
        self.inner.set_state(inner);
        self.outer.set_state(outer);
    }

    fn process(&self, input: A) -> Result<Self::Output, Self::Error> {
        let result: B = self.inner.process(input)?;
        self.outer.process(result.into())
    }
}
