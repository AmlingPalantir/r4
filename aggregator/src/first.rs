use record::Record;
use registry::args::OneKeyRegistryArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneKeyRegistryArgs;
    type State = Option<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["first"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("key");
    }

    fn help_msg() -> &'static str {
        return "track the first value";
    }

    fn add(state: &mut Option<Record>, a: &OneKeyRegistryArgs, r: Record) {
        state.get_or_insert(r.get_path(&a.key));
    }

    fn finish(state: Option<Record>, _a: &OneKeyRegistryArgs) -> Record {
        return state.unwrap();
    }
}
