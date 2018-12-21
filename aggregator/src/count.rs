use AggregatorBe;
use ZeroArgs;
use record::FromPrimitive;
use record::Record;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["ct", "count"];
}

#[derive(Default)]
pub struct Impl {
}

impl AggregatorBe for Impl {
    type Args = ZeroArgs;
    type State = u32;

    fn add(state: &mut u32, _a: &(), _r: Record) {
        *state += 1;
    }

    fn finish(state: u32, _a: &()) -> Record {
        return Record::from_primitive(state);
    }
}
