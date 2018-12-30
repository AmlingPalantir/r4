use record::Record;
use registry::args::ZeroArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = ZeroArgs;
    type State = Option<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["firstrecord", "firstrec"];
    }

    fn add(state: &mut Option<Record>, _a: &(), r: Record) {
        state.get_or_insert(r);
    }

    fn finish(state: Option<Record>, _a: &()) -> Record {
        return state.unwrap();
    }
}
