use record::Record;
use registry::args::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = Option<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["last"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("key");
    }

    fn help_msg() -> &'static str {
        return "track the last value";
    }

    fn add(state: &mut Option<Record>, a: &Arc<str>, r: Record) {
        *state = Some(r.get_path(a));
    }

    fn finish(state: Option<Record>, _a: &Arc<str>) -> Record {
        return state.unwrap();
    }
}
