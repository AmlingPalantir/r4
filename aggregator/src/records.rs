use AggregatorBe;
use record::Record;
use registry::ZeroArgs;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["recs", "records"];
}

#[derive(Default)]
pub struct Impl {
}

impl AggregatorBe for Impl {
    type Args = ZeroArgs;
    type State = Vec<Record>;

    fn add(state: &mut Vec<Record>, _a: &(), r: Record) {
        state.push(r);
    }

    fn finish(state: Vec<Record>, _a: &()) -> Record {
        return Record::from_vec(state);
    }
}
