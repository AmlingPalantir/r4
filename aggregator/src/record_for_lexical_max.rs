use record::Record;
use record::RecordTrait;
use registry::args::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::lexical_max::MaxState;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = MaxState<Arc<str>>;

    fn names() -> Vec<&'static str> {
        return vec!["recforlmax"];
    }

    fn add(state: &mut MaxState<Arc<str>>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(v.expect_string(), r);
    }

    fn finish(state: Box<MaxState<Arc<str>>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
