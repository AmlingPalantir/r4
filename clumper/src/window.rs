use ClumperBe;
use record::Record;
use registry::OneIntArgs;
use std::collections::VecDeque;
use std::sync::Arc;
use stream::Entry;
use stream::Flow;
use stream::Stream;

pub struct Impl();

impl ClumperBe for Impl {
    type Args = OneIntArgs;

    fn names() -> Vec<&'static str> {
        return vec!["window"];
    }

    fn stream(size: &i64, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        let size = *size;

        return stream::compound(
            stream::parse(),
            stream::closures(
                VecDeque::new(),
                move |s, e, w| {
                    match e {
                        Entry::Bof(file) => {
                            s.clear();
                            return w(Entry::Bof(file));
                        },
                        Entry::Record(r) => {
                            s.push_back(r);
                            if s.len() as i64 > size {
                                s.pop_front();
                            }
                            if s.len() as i64 == size {
                                let mut substream = bsw(vec![]);

                                for r in s {
                                    // Disregard flow since one substream
                                    // ending does not mean we're done (e.g.
                                    // each substream could be head -n 1).
                                    substream.write(Entry::Record(r.clone()), w);
                                }

                                Box::new(substream).close(w);
                            }
                            return Flow(true);
                        },
                        Entry::Line(_line) => {
                            panic!("Unexpected line in WindowStream");
                        },
                    }
                },
                |_s, _w| {
                },
            ),
        );
    }
}
