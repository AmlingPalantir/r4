use record::Record;
use registry::ZeroArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl AggregatorBe for ImplBe {
    type Args = ZeroArgs;
    type State = i64;

    fn names() -> Vec<&'static str> {
        return vec!["ct", "count"];
    }

    fn add(state: &mut i64, _a: &(), _r: Record) {
        *state += 1;
    }

    fn finish(state: Box<i64>, _a: &()) -> Record {
        return Record::from(*state);
    }
}
