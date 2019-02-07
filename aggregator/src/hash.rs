use record::Record;
use record::RecordTrait;
use std::collections::BTreeMap;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

#[derive(RegistryArgs)]
pub(crate) struct Args {
    key_key: Arc<str>,
    value_key: Arc<str>,
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = Args;
    type State = BTreeMap<Arc<str>, Record>;

    fn names() -> Vec<&'static str> {
        return vec!["hash"];
    }

    fn help_msg() -> &'static str {
        return "collect pairs (key and value) of values into a hash";
    }

    fn add(state: &mut BTreeMap<Arc<str>, Record>, a: &Args, r: Record) {
        state.insert(r.get_path(&a.key_key).expect_string(), r.get_path(&a.value_key));
    }

    fn finish(state: BTreeMap<Arc<str>, Record>, _a: &Args) -> Record {
        return Record::from_hash(state);
    }
}
