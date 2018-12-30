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
        return vec!["first"];
    }

    fn add(state: &mut Option<Record>, a: &Arc<str>, r: Record) {
        state.get_or_insert(r.get_path(a));
    }

    fn finish(state: Option<Record>, _a: &Arc<str>) -> Record {
        return state.unwrap();
    }
}
