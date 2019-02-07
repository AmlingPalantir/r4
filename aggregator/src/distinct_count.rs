use record::Record;
use registry::args::OneKeyRegistryArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::distinct_array::DistinctSet;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneKeyRegistryArgs;
    type State = DistinctSet<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["dcount", "dct"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("key");
    }

    fn help_msg() -> &'static str {
        return "count distinct values";
    }

    fn add(state: &mut DistinctSet<Record>, a: &OneKeyRegistryArgs, r: Record) {
        state.add(r.get_path(&a.key));
    }

    fn finish(state: DistinctSet<Record>, _a: &OneKeyRegistryArgs) -> Record {
        return Record::from(state.into_iter().count() as i64);
    }
}
