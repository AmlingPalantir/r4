use record::Record;
use registry::OneUsizeArgs;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::ClumperBe;
use super::ClumperRegistrant;

pub type Impl = ClumperRegistrant<ImplBe>;

pub struct ImplBe();

impl ClumperBe for ImplBe {
    type Args = OneUsizeArgs;

    fn names() -> Vec<&'static str> {
        return vec!["rr", "round-robin"];
    }

    fn stream(n: &usize, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        let n = *n;
        let substreams: Vec<_> = (0..n).map(|_| bsw(vec![])).collect();

        return stream::closures(
            (substreams, 0),
            move |s, e, w| {
                match e {
                    Entry::Bof(_file) => {
                        return true;
                    },
                    e => {
                        let i = s.1;
                        let i = (i + 1) % s.0.len();
                        s.1 = i;

                        // Again, substream ending does not concern us, we may
                        // need to truck on for other streams.
                        s.0[i].write(e, w);

                        return true;
                    },
                }
            },
            |s, w| {
                for substream in s.0.into_iter() {
                    substream.close(w);
                }
            },
        );
    }
}
