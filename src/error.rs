use serde::export::fmt::Debug;
use std::any::Any;

#[derive(thiserror::Error, Debug)]
pub enum Er {
    #[error("err")]
    E,
}

#[derive(thiserror::Error, Debug)]
pub enum RunError<S>
where
    S: Debug + 'static,
{
    #[error("threading failure")]
    Threading(Box<dyn Any + Send>),

    #[error("")]
    Source(S),

    #[error("")]
    Processor(S),
}
