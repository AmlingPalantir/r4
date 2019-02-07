use record::Record;
use registry::args::ZeroRegistryArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = ZeroRegistryArgs;
    type State = Option<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["lastrecord", "lastrec"];
    }

    fn help_msg() -> &'static str {
        return "track last record";
    }

    fn add(state: &mut Option<Record>, _a: &ZeroRegistryArgs, r: Record) {
        *state = Some(r);
    }

    fn finish(state: Option<Record>, _a: &ZeroRegistryArgs) -> Record {
        return state.unwrap();
    }
}
