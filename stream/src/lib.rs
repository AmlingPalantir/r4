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
    fn bof(&mut self, &str);
    fn write(&mut self, Entry);
    fn rclosed(&mut self) -> bool;
    fn close(&mut self);
}

pub struct Stream(Box<StreamTrait>);

impl Stream {
    pub fn new<F: StreamTrait + 'static>(f: F) -> Self {
        return Stream(Box::new(f));
    }

    pub fn transform_entries<F: FnMut(Entry) -> Entry + 'static>(self, f: F) -> Stream {
        return Stream::new(TransformStream {
            os: self,
            f: Box::new(f),
        });
    }
}

impl StreamTrait for Stream {
    fn bof(&mut self, file: &str) {
        self.0.bof(file);
    }

    fn write(&mut self, e: Entry) {
        self.0.write(e);
    }

    fn rclosed(&mut self) -> bool {
        return self.0.rclosed();
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
    fn bof(&mut self, file: &str) {
        self.os.bof(file);
    }

    fn write(&mut self, e: Entry) {
        self.os.write((*self.f)(e));
    }

    fn rclosed(&mut self) -> bool {
        return self.os.rclosed();
    }

    fn close(&mut self) {
        self.os.close();
    }
}
