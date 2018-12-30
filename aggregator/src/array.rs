use record::Record;
use record::RecordTrait;
use registry::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = Vec<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["arr", "array"];
    }

    fn add(state: &mut Vec<Record>, a: &Arc<str>, r: Record) {
        state.push(r.get_path(a));
    }

    fn finish(state: Box<Vec<Record>>, _a: &Arc<str>) -> Record {
        return Record::from_vec(*state);
    }
}
