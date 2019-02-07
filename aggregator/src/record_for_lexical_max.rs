use record::Record;
use record::RecordTrait;
use registry_args::OneStringArgs;
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

    fn help_meta() -> Option<&'static str> {
        return Some("key");
    }

    fn help_msg() -> &'static str {
        return "track the record for the lexically maximal value";
    }

    fn add(state: &mut MaxState<Arc<str>>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(v.expect_string(), r);
    }

    fn finish(state: MaxState<Arc<str>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
