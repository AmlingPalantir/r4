use ClumperBe;
use record::Record;
use registry::OneStringArgs;
use std::sync::Arc;
use stream::Stream;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["k", "key"];
}

#[derive(Default)]
pub struct Impl {
}

impl ClumperBe for Impl {
    type Args = OneStringArgs;

    fn wrap(k: &Arc<str>, os: Stream, bsw: Box<Fn(Stream, Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        return os;
    }
}
