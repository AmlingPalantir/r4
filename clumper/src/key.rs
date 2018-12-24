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

    fn wrap(k: &Arc<str>, os: Stream, bsw: Box<Fn(Stream, Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        return Stream::new(KeyStream {
            k: k.clone(),
            os: os,
            bsw: bsw,
            substreams: HashMap::new(),
        }).parse();
    }
}

struct KeyStream {
    k: Arc<str>,
    os: Stream,
    bsw: Box<Fn(Stream, Vec<(Arc<str>, Record)>) -> Stream>,
    substreams: HashMap<Record, Stream>,
}

impl KeyStream {
    fn find_stream(&mut self, v: Record) -> &mut Stream {
        return self.substreams.entry(v).or_insert_with(|| {
        });
    }
}

impl StreamTrait for KeyStream {
    fn write(&mut self, e: Entry) {
        match e {
            Entry::Bof(_file) => {
            },
            Entry::Record(r) => {
                self.find_stream(r.get_path(&self.k)).write(Entry::Record(r));
            },
            Entry::Line(_line) => {
                panic!();
            },
        }
    }

    fn close(self: Box<KeyStream>) {
        for (_, substream) in self.substreams.into_iter() {
            substream.close();
        }
        self.os.close();
    }

    fn rclosed(&mut self) -> bool {
        return self.os.rclosed();
    }
}
