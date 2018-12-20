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

pub trait StreamTrait {
    fn write(&mut self, Entry) -> bool;
    fn close(&mut self);
}

pub struct Stream(Box<StreamTrait>);

impl Stream {
    pub fn new<F: StreamTrait + 'static>(f: F) -> Self {
        return Stream(Box::new(f));
    }
}

impl StreamTrait for Stream {
    fn write(&mut self, e: Entry) -> bool {
        return self.0.write(e);
    }

    fn close(&mut self) {
        self.0.close();
    }
}

struct TransformStream {
    os: Stream,
    f: Box<FnMut(Entry) -> Entry>,
}

impl StreamTrait for TransformStream {
    fn write(&mut self, e: Entry) -> bool {
        return self.os.write((*self.f)(e));
    }

    fn close(&mut self) {
        self.os.close();
    }
}

pub fn transform<F: FnMut(Entry) -> Entry + 'static>(os: Stream, f: F) -> Stream {
    return Stream::new(TransformStream {
        os: os,
        f: Box::new(f),
    });
}
