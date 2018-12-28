use AggregatorBe;
use record::Record;
use record::RecordTrait;
use registry::OneStringArgs;
use std::sync::Arc;
use super::lexical_max::MaxState;
use super::lexical_min::ReverseOrd;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = MaxState<ReverseOrd<Arc<str>>>;

    fn names() -> Vec<&'static str> {
        return vec!["recforlmin"];
    }

    fn add(state: &mut MaxState<ReverseOrd<Arc<str>>>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(ReverseOrd(v.expect_string()), r);
    }

    fn finish(state: Box<MaxState<ReverseOrd<Arc<str>>>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
