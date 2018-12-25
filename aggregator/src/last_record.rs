use AggregatorBe;
use record::Record;
use registry::ZeroArgs;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = ZeroArgs;
    type State = Option<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["lastrecord", "lastrec"];
    }

    fn add(state: &mut Option<Record>, _a: &(), r: Record) {
        *state = Some(r);
    }

    fn finish(state: Box<Option<Record>>, _a: &()) -> Record {
        return state.unwrap();
    }
}
