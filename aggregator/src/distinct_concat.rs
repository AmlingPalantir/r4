use AggregatorBe;
use record::Record;
use record::RecordTrait;
use registry::TwoStringArgs;
use std::sync::Arc;
use super::distinct_array::DistinctSet;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = TwoStringArgs;
    type State = DistinctSet<String>;

    fn names() -> Vec<&'static str> {
        return vec!["dconcatenate", "dconcat"];
    }

    fn add(state: &mut DistinctSet<String>, a: &(Arc<str>, Arc<str>), r: Record) {
        state.add(r.get_path(&a.1).expect_string().to_string());
    }

    fn finish(state: Box<DistinctSet<String>>, a: &(Arc<str>, Arc<str>)) -> Record {
        let vs: Vec<String> = state.into_iter().collect();
        return Record::from(vs.join(&a.1));
    }
}
