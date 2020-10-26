use crossbeam_queue::ArrayQueue;

pub enum State {
    Submitted,
    Processed,
}

pub trait Broker<T> {
    fn submit(&self, value: T) -> Result<(), T>;
    fn claim(&self) -> Option<T>;
}

impl<T> Broker<T> for ArrayQueue<T> {
    fn submit(&self, value: T) -> Result<(), T> {
        self.push(value)
    }
    fn claim(&self) -> Option<T> {
        self.pop()
    }
}
