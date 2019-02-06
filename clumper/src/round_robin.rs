use record::Record;
use registry::args::OneUsizeArgs;
use std::sync::Arc;
use stream::Stream;
use super::ClumperBe;
use super::ClumperRegistrant;

pub type Impl = ClumperRegistrant<ImplBe>;

pub struct ImplBe();

impl ClumperBe for ImplBe {
    type Args = OneUsizeArgs;

    fn names() -> Vec<&'static str> {
        return vec!["round-robin", "rr"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("count");
    }

    fn help_msg() -> &'static str {
        return "bucket records rotating between a specified number of buckets";
    }

    fn stream(n: &usize, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        let n = *n;
        let substreams: Vec<_> = (0..n).map(|_| bsw(vec![])).collect();

        return stream::closures(
            (substreams, 0),
            |s, e, w| {
                let i = s.1;
                let i = (i + 1) % s.0.len();
                s.1 = i;

                // Again, substream ending does not concern us, we may
                // need to truck on for other streams.
                s.0[i].write(e, w);

                return true;
            },
            |s, w| {
                for substream in s.0.into_iter() {
                    substream.close(w);
                }
            },
        );
    }
}
