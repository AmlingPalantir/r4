use record::Record;
use registry::OneStringArgs;
use std::collections::HashMap;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::ClumperBe;
use super::ClumperRegistrant;

pub(crate) type Impl = ClumperRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl ClumperBe for ImplBe {
    type Args = OneStringArgs;

    fn names() -> Vec<&'static str> {
        return vec!["k", "key"];
    }

    fn stream(k: &Arc<str>, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        let k = k.clone();

        return stream::compound(
            stream::parse(),
            stream::closures(
                HashMap::new(),
                move |s, e, w| {
                    match e {
                        Entry::Bof(_file) => {
                            return true;
                        },
                        Entry::Record(r) => {
                            let v = r.get_path(&k);

                            let substream = s.entry(v.clone()).or_insert_with(|| {
                                return bsw(vec![(k.clone(), v)]);
                            });

                            // Disregard flow since one substream ending does
                            // not mean we're done (e.g.  each substream could
                            // be head -n 1).
                            substream.write(Entry::Record(r), w);

                            return true;
                        },
                        Entry::Line(_line) => {
                            panic!("Unexpected line in KeyStream");
                        },
                    }
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
