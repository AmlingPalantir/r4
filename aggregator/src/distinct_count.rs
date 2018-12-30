use record::Record;
use registry::args::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::distinct_array::DistinctSet;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = DistinctSet<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["dcount", "dct"];
    }

    fn add(state: &mut DistinctSet<Record>, a: &Arc<str>, r: Record) {
        state.add(r.get_path(&a));
    }

    fn finish(state: Box<DistinctSet<Record>>, _a: &Arc<str>) -> Record {
        return Record::from(state.into_iter().count() as i64);
    }
}
