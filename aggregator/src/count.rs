use AggregatorBe;
use record::Record;
use registry::ZeroArgs;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = ZeroArgs;
    type State = i64;

    fn names() -> Vec<&'static str> {
        return vec!["ct", "count"];
    }

    fn add(state: &mut i64, _a: &(), _r: Record) {
        *state += 1;
    }

    fn finish(state: Box<i64>, _a: &()) -> Record {
        return Record::from_i64(*state);
    }
}
