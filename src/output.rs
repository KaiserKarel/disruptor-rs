use crate::log::Entry;

pub struct Output<'a, T> {
    pub result: T,
    pub log: Vec<Entry<'a>>,
}
