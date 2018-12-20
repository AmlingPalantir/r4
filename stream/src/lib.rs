extern crate record;

use record::Record;
use std::str::FromStr;
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

    pub fn to_record(&self) -> Record {
        return match self {
            Entry::Record(r) => r.clone(),
            Entry::Line(s) => Record::from_str(s).unwrap(),
        }
    }
}

pub trait Stream {
    fn write(&mut self, Entry) -> bool;
    fn close(&mut self);
}

struct TransformStream {
    os: Box<Stream>,
    f: Box<FnMut(Entry) -> Entry>,
}

impl Stream for TransformStream {
    fn write(&mut self, e: Entry) -> bool {
        return self.os.write((*self.f)(e));
    }

    fn close(&mut self) {
        self.os.close();
    }
}

pub fn transform<F: FnMut(Entry) -> Entry + 'static>(os: Box<Stream>, f: F) -> Box<Stream> {
    return Box::new(TransformStream {
        os: os,
        f: Box::new(f),
    });
}
