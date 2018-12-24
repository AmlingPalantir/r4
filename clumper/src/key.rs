use ClumperBe;
use record::Record;
use registry::OneStringArgs;
use std::collections::HashMap;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use stream::StreamTrait;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["k", "key"];
}

#[derive(Default)]
pub struct Impl {
}

impl ClumperBe for Impl {
    type Args = OneStringArgs;

    fn stream(k: &Arc<str>, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        let s = Stream::new(KeyStream {
            k: k.clone(),
            bsw: bsw,
            substreams: HashMap::new(),
        });
        return Stream::compound(Stream::parse(), s);
    }
}

struct KeyStream {
    k: Arc<str>,
    bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>,
    substreams: HashMap<Record, Stream>,
}

impl KeyStream {
    fn find_stream(&mut self, v: Record) -> &mut Stream {
        let k = &self.k;
        let bsw = &self.bsw;
        return self.substreams.entry(v.clone()).or_insert_with(|| {
            return bsw(vec![(k.clone(), v)]);
        });
    }
}

impl StreamTrait for KeyStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        match e {
            Entry::Bof(_file) => {
            },
            Entry::Record(r) => {
                let v = r.get_path(&self.k);
                self.find_stream(v).write(Entry::Record(r), w);
            },
            Entry::Line(_line) => {
                panic!();
            },
        }
        // Sad, but you could always be opening a new stream so we can never be
        // sure we're done.
        return true;
    }

    fn close(self: Box<KeyStream>, w: &mut FnMut(Entry) -> bool) {
        for (_, substream) in self.substreams.into_iter() {
            substream.close(w);
        }
    }
}
