pub trait Store<T> {
    type ID;
    fn ack(&self, id: Self::ID) -> Result<(), std::io::Error>;
    fn insert(&self, value: T) -> Result<Self::ID, std::io::Error>;
}
