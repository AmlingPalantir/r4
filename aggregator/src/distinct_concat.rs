use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::distinct_array::DistinctSet;

#[derive(RegistryArgs)]
pub(crate) struct Args {
    delimiter: Arc<str>,
    key: Arc<str>,
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = Args;
    type State = DistinctSet<String>;

    fn names() -> Vec<&'static str> {
        return vec!["dconcatenate", "dconcat"];
    }

    fn help_msg() -> &'static str {
        return "collect distinct values into a string joined by a delimter";
    }

    fn add(state: &mut DistinctSet<String>, a: &Args, r: Record) {
        state.add(r.get_path(&a.key).expect_string().to_string());
    }

    fn finish(state: DistinctSet<String>, a: &Args) -> Record {
        let vs: Vec<_> = state.into_iter().collect();
        return Record::from(vs.join(&a.delimiter));
    }
}
