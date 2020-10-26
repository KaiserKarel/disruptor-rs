#[derive(Debug, PartialEq)]
pub struct Entry<'a> {
    pub(crate) text: &'a str,
}

impl Entry<'_> {
    pub fn new(text: &str) -> Entry {
        Entry { text }
    }
}
