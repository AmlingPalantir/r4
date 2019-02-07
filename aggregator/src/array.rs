use record::Record;
use record::RecordTrait;
use registry::args::OneKeyRegistryArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl AggregatorBe for ImplBe {
    type Args = OneKeyRegistryArgs;
    type State = Vec<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["array", "arr"];
    }

    fn help_msg() -> &'static str {
        return "collect values into an array";
    }

    fn add(state: &mut Vec<Record>, a: &OneKeyRegistryArgs, r: Record) {
        state.push(r.get_path(&a.key));
    }

    fn finish(state: Vec<Record>, _a: &OneKeyRegistryArgs) -> Record {
        return Record::from_vec(state);
    }
}
