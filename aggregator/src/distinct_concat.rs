use record::Record;
use record::RecordTrait;
use registry::args::TwoStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::distinct_array::DistinctSet;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = TwoStringArgs;
    type State = DistinctSet<String>;

    fn names() -> Vec<&'static str> {
        return vec!["dconcatenate", "dconcat"];
    }

    fn add(state: &mut DistinctSet<String>, a: &(Arc<str>, Arc<str>), r: Record) {
        state.add(r.get_path(&a.1).expect_string().to_string());
    }

    fn finish(state: DistinctSet<String>, a: &(Arc<str>, Arc<str>)) -> Record {
        let vs: Vec<_> = state.into_iter().collect();
        return Record::from(vs.join(&a.1));
    }
}
