extern crate record;

use record::Record;
use std::sync::Arc;

#[derive(Clone)]
pub enum Entry {
    Record(Record),
    Line(Arc<str>),
}

impl Entry {
    pub fn to_line(&self) -> Arc<str> {
        return match self {
            Entry::Record(r) => Arc::from(r.to_string()),
            Entry::Line(s) => s.clone(),
        }
    }
}

pub trait Stream {
    fn write(&mut self, Entry) -> bool;
    fn close(&mut self);
}
