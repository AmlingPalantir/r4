use record::Record;
use record::RecordTrait;
use registry_args::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = Vec<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["array", "arr"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("key");
    }

    fn help_msg() -> &'static str {
        return "collect values into an array";
    }

    fn add(state: &mut Vec<Record>, a: &Arc<str>, r: Record) {
        state.push(r.get_path(a));
    }

    fn finish(state: Vec<Record>, _a: &Arc<str>) -> Record {
        return Record::from_vec(state);
    }
}
