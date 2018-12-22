extern crate record;

use record::Record;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub enum Entry {
    Bof(Arc<str>),
    Record(Record),
    Line(Arc<str>),
}

pub trait StreamTrait {
    fn write(&mut self, Entry);
    fn close(self: Box<Self>);
    fn rclosed(&mut self) -> bool;
}

pub struct Stream(Box<StreamTrait>);

impl Stream {
    pub fn new<F: StreamTrait + 'static>(f: F) -> Self {
        return Stream(Box::new(f));
    }

    pub fn parse(self) -> Stream {
        return Stream::new(ParseStream(self));
    }

    pub fn transform_records<F: FnMut(Record) -> Record + 'static>(self, f: F) -> Stream {
        return Stream::new(TransformRecordsStream {
            os: self,
            f: Box::new(f),
        });
    }
}

impl Stream {
    pub fn write(&mut self, e: Entry) {
        self.0.write(e);
    }

    pub fn close(self) {
        self.0.close();
    }

    pub fn rclosed(&mut self) -> bool {
        return self.0.rclosed();
    }
}

struct ParseStream(Stream);

impl StreamTrait for ParseStream {
    fn write(&mut self, e: Entry) {
        match e {
            Entry::Line(line) => {
                self.0.write(Entry::Record(Record::from_str(&line).unwrap()));
            }
            e => {
                self.0.write(e);
            }
        }
    }

    fn close(self: Box<ParseStream>) {
        self.0.close();
    }

    fn rclosed(&mut self) -> bool {
        return self.0.rclosed();
    }
}

struct TransformRecordsStream {
    os: Stream,
    f: Box<FnMut(Record) -> Record>,
}

impl StreamTrait for TransformRecordsStream {
    fn write(&mut self, e: Entry) {
        match e {
            Entry::Record(r) => {
                self.os.write(Entry::Record((*self.f)(r)));
            }
            e => {
                self.os.write(e);
            }
        }
    }

    fn close(self: Box<TransformRecordsStream>) {
        self.os.close();
    }

    fn rclosed(&mut self) -> bool {
        return self.os.rclosed();
    }
}
