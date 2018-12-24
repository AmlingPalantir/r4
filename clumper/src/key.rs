use ClumperBe;
use record::Record;
use registry::OneStringArgs;
use std::collections::HashMap;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
pub(crate) fn names() -> Vec<&'static str> {
    return vec!["k", "key"];
}

#[derive(Default)]
pub struct Impl {
}

impl ClumperBe for Impl {
    type Args = OneStringArgs;

    fn stream(k: &Arc<str>, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        let k = k.clone();

        return stream::compound(
            stream::parse(),
            stream::closures(
                HashMap::new(),
                move |s, e, w| {
                    match e {
                        Entry::Bof(_file) => {
                        },
                        Entry::Record(r) => {
                            let v = r.get_path(&k);

                            let substream = s.entry(v.clone()).or_insert_with(|| {
                                return bsw(vec![(k.clone(), v)]);
                            });

                            substream.write(Entry::Record(r), w);
                        },
                        Entry::Line(_line) => {
                            panic!("Unexpected line in KeyStream");
                        },
                    }
                    // Sad, but you could always be opening a new stream so we can never be
                    // sure we're done.
                    return true;
                },
                |s, w| {
                    for (_, substream) in s.into_iter() {
                        substream.close(w);
                    }
                },
            ),
        );
    }
}
