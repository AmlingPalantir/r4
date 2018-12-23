use AggregatorBe;
use record::FromPrimitive;
use record::Record;
use registry::ZeroArgs;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["ct", "count"];
}

#[derive(Default)]
pub struct Impl {
}

impl AggregatorBe for Impl {
    type Args = ZeroArgs;
    type State = i64;

    fn add(state: &mut i64, _a: &(), _r: Record) {
        *state += 1;
    }

    fn finish(state: Box<i64>, _a: &()) -> Record {
        return Record::from_primitive(*state);
    }
}
