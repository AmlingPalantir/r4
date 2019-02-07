use record::Record;
use record::RecordTrait;
use registry::args::OneKeyRegistryArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::lexical_max::MaxState;
use super::lexical_min::ReverseOrd;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneKeyRegistryArgs;
    type State = MaxState<ReverseOrd<Arc<str>>>;

    fn names() -> Vec<&'static str> {
        return vec!["recforlmin"];
    }

    fn help_msg() -> &'static str {
        return "track the record for the lexically minimal value";
    }

    fn add(state: &mut MaxState<ReverseOrd<Arc<str>>>, a: &OneKeyRegistryArgs, r: Record) {
        let v = r.get_path(&a.key);
        state.add(ReverseOrd(v.expect_string()), r);
    }

    fn finish(state: MaxState<ReverseOrd<Arc<str>>>, _a: &OneKeyRegistryArgs) -> Record {
        return state.finish();
    }
}
