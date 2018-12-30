use record::Record;
use record::RecordTrait;
use registry::args::ZeroArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = ZeroArgs;
    type State = Vec<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["recs", "records"];
    }

    fn add(state: &mut Vec<Record>, _a: &(), r: Record) {
        state.push(r);
    }

    fn finish(state: Box<Vec<Record>>, _a: &()) -> Record {
        return Record::from_vec(*state);
    }
}
