use AggregatorBe;
use record::Record;
use registry::ZeroArgs;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = ZeroArgs;
    type State = Option<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["firstrecord", "firstrec"];
    }

    fn add(state: &mut Option<Record>, _a: &(), r: Record) {
        state.get_or_insert(r);
    }

    fn finish(state: Box<Option<Record>>, _a: &()) -> Record {
        return state.unwrap();
    }
}
