use record::Record;
use registry::args::ZeroRegistryArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl AggregatorBe for ImplBe {
    type Args = ZeroRegistryArgs;
    type State = i64;

    fn names() -> Vec<&'static str> {
        return vec!["count", "ct"];
    }

    fn help_msg() -> &'static str {
        return "count records";
    }

    fn add(state: &mut i64, _a: &ZeroRegistryArgs, _r: Record) {
        *state += 1;
    }

    fn finish(state: i64, _a: &ZeroRegistryArgs) -> Record {
        return Record::from(state);
    }
}
