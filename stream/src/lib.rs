extern crate record;

use record::Record;
use std::sync::Arc;

#[derive(Clone)]
pub enum Entry {
    Bof(Arc<str>),
    Record(Record),
    Line(Arc<str>),
}

pub trait StreamTrait {
    fn write(&mut self, r: Entry, w: &mut FnMut(Entry) -> bool) -> bool;
    fn close(self: Box<Self>, w: &mut FnMut(Entry) -> bool);
}

pub struct Stream(Box<StreamTrait>);

impl Stream {
    pub fn new<F: StreamTrait + 'static>(f: F) -> Self {
        return Stream(Box::new(f));
    }

    pub fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        let mut ret = true;
        let ret2 = self.0.write(e, &mut |e| {
            let r = w(e);
            if !r {
                ret = false;
            }
            return r;
        });
        return ret && ret2;
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

pub fn id() -> Stream {
    return Stream::new(IdStream());
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

pub fn compound(s1: Stream, s2: Stream) -> Stream {
    return Stream::new(CompoundStream(s1, s2));
}

struct ParseStream();

impl StreamTrait for ParseStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return w(match e {
            Entry::Line(line) => Entry::Record(Record::parse(&line)),
            e => e,
        });
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> bool) {
    }
}

pub fn parse() -> Stream {
    return Stream::new(ParseStream());
}

struct DeparseStream();

impl StreamTrait for DeparseStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return w(match e {
            Entry::Record(r) => Entry::Line(Arc::from(r.deparse())),
            e => e,
        });
    }

    fn close(self: Box<Self>, _w: &mut FnMut(Entry) -> bool) {
    }
}

pub fn deparse() -> Stream {
    return Stream::new(DeparseStream());
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

pub fn transform_records<F: FnMut(Record) -> Record + 'static>(f: F) -> Stream {
    return Stream::new(TransformRecordsStream(Box::new(f)));
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

pub fn drop_bof() -> Stream {
    return Stream::new(DropBofStream());
}

struct ClosuresStream<S>(S, Box<Fn(&mut S, Entry, &mut FnMut(Entry) -> bool) -> bool>, Box<Fn(Box<S>, &mut FnMut(Entry) -> bool)>);

impl<S> StreamTrait for ClosuresStream<S> {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return self.1(&mut self.0, e, w);
    }

    fn close(self: Box<Self>, w: &mut FnMut(Entry) -> bool) {
        let s = *self;
        s.2(Box::new(s.0), w);
    }
}

pub fn closures<S: 'static, W: Fn(&mut S, Entry, &mut FnMut(Entry) -> bool) -> bool + 'static, C: Fn(Box<S>, &mut FnMut(Entry) -> bool) + 'static>(s: S, w: W, c: C) -> Stream {
    return Stream::new(ClosuresStream(s, Box::new(w), Box::new(c)));
}
