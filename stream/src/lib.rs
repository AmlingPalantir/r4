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
    fn write(&mut self, Entry, &mut FnMut(Entry) -> bool) -> bool;
    fn close(self: Box<Self>, &mut FnMut(Entry) -> bool);
}

pub struct Stream(Box<StreamTrait>);

impl Stream {
    pub fn new<F: StreamTrait + 'static>(f: F) -> Self {
        return Stream(Box::new(f));
    }

    pub fn id() -> Stream {
        return Stream::new(IdStream());
    }

    pub fn compound(s1: Stream, s2: Stream) -> Stream {
        return Stream::new(CompoundStream(s1, s2));
    }

    pub fn parse() -> Stream {
        return Stream::new(ParseStream());
    }

    pub fn transform_records<F: FnMut(Record) -> Record + 'static>(f: F) -> Stream {
        return Stream::new(TransformRecordsStream(Box::new(f)));
    }

    pub fn drop_bof() -> Stream {
        return Stream::new(DropBofStream());
    }
}

impl Stream {
    pub fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return self.0.write(e, w);
    }

    pub fn close(self, w: &mut FnMut(Entry) -> bool) {
        self.0.close(w);
    }
}

struct IdStream();

impl StreamTrait for IdStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return w(e);
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> bool) {
    }
}

struct CompoundStream(Stream, Stream);

impl StreamTrait for CompoundStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        let s2 = &mut self.1;
        return self.0.write(e, &mut |e| s2.write(e, w));
    }

    fn close(self: Box<Self>, w: &mut FnMut(Entry) -> bool) {
        let s = *self;
        let mut s2 = s.1;
        s.0.close(&mut |e| s2.write(e, w));
        s2.close(w);
    }
}

struct ParseStream();

impl StreamTrait for ParseStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return w(match e {
            Entry::Line(line) => Entry::Record(Record::from_str(&line).unwrap()),
            e => e,
        });
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> bool) {
    }
}

struct TransformRecordsStream(Box<FnMut(Record) -> Record>);

impl StreamTrait for TransformRecordsStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return w(match e {
            Entry::Record(r) => Entry::Record((*self.0)(r)),
            e => e,
        });
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> bool) {
    }
}

struct DropBofStream();

impl StreamTrait for DropBofStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return match e {
            Entry::Bof(_file) => true,
            e => w(e),
        };
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> bool) {
    }
}
