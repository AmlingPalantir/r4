extern crate record;

use record::Record;
use std::sync::Arc;

#[derive(Clone)]
pub enum Entry {
    Bof(Arc<str>),
    Record(Record),
    Line(Arc<str>),
}

pub struct Flow(pub bool);

pub trait StreamTrait {
    fn write(&mut self, Entry, &mut FnMut(Entry) -> Flow) -> Flow;
    fn close(self: Box<Self>, &mut FnMut(Entry) -> Flow);
}

pub struct Stream(Box<StreamTrait>);

impl Stream {
    pub fn new<F: StreamTrait + 'static>(f: F) -> Self {
        return Stream(Box::new(f));
    }

    pub fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> Flow) -> Flow {
        let mut ret = true;
        let ret2 = self.0.write(e, &mut |e| {
            let r = w(e);
            if !r.0 {
                ret = false;
            }
            return r;
        });
        return Flow(ret && ret2.0);
    }

    pub fn close(self, w: &mut FnMut(Entry) -> Flow) {
        self.0.close(w);
    }
}

struct IdStream();

impl StreamTrait for IdStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> Flow) -> Flow {
        return w(e);
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> Flow) {
    }
}

pub fn id() -> Stream {
    return Stream::new(IdStream());
}

struct CompoundStream(Stream, Stream);

impl StreamTrait for CompoundStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> Flow) -> Flow {
        let s2 = &mut self.1;
        return self.0.write(e, &mut |e| s2.write(e, w));
    }

    fn close(self: Box<Self>, w: &mut FnMut(Entry) -> Flow) {
        let s = *self;
        let mut s2 = s.1;
        s.0.close(&mut |e| s2.write(e, w));
        s2.close(w);
    }
}

pub fn compound(s1: Stream, s2: Stream) -> Stream {
    return Stream::new(CompoundStream(s1, s2));
}

struct ParseStream();

impl StreamTrait for ParseStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> Flow) -> Flow {
        return w(match e {
            Entry::Line(line) => Entry::Record(Record::parse(&line)),
            e => e,
        });
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> Flow) {
    }
}

pub fn parse() -> Stream {
    return Stream::new(ParseStream());
}

struct DeparseStream();

impl StreamTrait for DeparseStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> Flow) -> Flow {
        return w(match e {
            Entry::Record(r) => Entry::Line(Arc::from(r.deparse())),
            e => e,
        });
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> Flow) {
    }
}

pub fn deparse() -> Stream {
    return Stream::new(DeparseStream());
}

struct TransformRecordsStream(Box<FnMut(Record) -> Record>);

impl StreamTrait for TransformRecordsStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> Flow) -> Flow {
        return w(match e {
            Entry::Record(r) => Entry::Record((*self.0)(r)),
            e => e,
        });
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> Flow) {
    }
}

pub fn transform_records<F: FnMut(Record) -> Record + 'static>(f: F) -> Stream {
    return Stream::new(TransformRecordsStream(Box::new(f)));
}

struct DropBofStream();

impl StreamTrait for DropBofStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> Flow) -> Flow {
        return match e {
            Entry::Bof(_file) => Flow(true),
            e => w(e),
        };
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> Flow) {
    }
}

pub fn drop_bof() -> Stream {
    return Stream::new(DropBofStream());
}

struct ClosuresStream<S>(S, Box<Fn(&mut S, Entry, &mut FnMut(Entry) -> Flow) -> Flow>, Box<Fn(Box<S>, &mut FnMut(Entry) -> Flow)>);

impl<S> StreamTrait for ClosuresStream<S> {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> Flow) -> Flow {
        return self.1(&mut self.0, e, w);
    }

    fn close(self: Box<Self>, w: &mut FnMut(Entry) -> Flow) {
        let s = *self;
        s.2(Box::new(s.0), w);
    }
}

pub fn closures<S: 'static, W: Fn(&mut S, Entry, &mut FnMut(Entry) -> Flow) -> Flow + 'static, C: Fn(Box<S>, &mut FnMut(Entry) -> Flow) + 'static>(s: S, w: W, c: C) -> Stream {
    return Stream::new(ClosuresStream(s, Box::new(w), Box::new(c)));
}
