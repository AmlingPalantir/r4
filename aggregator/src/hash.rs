use record::Record;
use record::RecordTrait;
use registry_args::TwoStringArgs;
use std::collections::BTreeMap;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = TwoStringArgs;
    type State = BTreeMap<Arc<str>, Record>;

    fn names() -> Vec<&'static str> {
        return vec!["hash"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("key_key,value_key");
    }

    fn help_msg() -> &'static str {
        return "collect pairs (key and value) of values into a hash";
    }

    fn add(state: &mut BTreeMap<Arc<str>, Record>, a: &(Arc<str>, Arc<str>), r: Record) {
        state.insert(r.get_path(&a.0).expect_string(), r.get_path(&a.1));
    }

    fn finish(state: BTreeMap<Arc<str>, Record>, _a: &(Arc<str>, Arc<str>)) -> Record {
        return Record::from_hash(state);
    }
}
