use record::Record;
use registry::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = Option<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["last"];
    }

    fn add(state: &mut Option<Record>, a: &Arc<str>, r: Record) {
        *state = Some(r.get_path(a));
    }

    fn finish(state: Box<Option<Record>>, _a: &Arc<str>) -> Record {
        return state.unwrap();
    }
}
