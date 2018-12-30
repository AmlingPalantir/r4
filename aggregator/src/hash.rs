use record::Record;
use record::RecordTrait;
use registry::args::TwoStringArgs;
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

    fn add(state: &mut BTreeMap<Arc<str>, Record>, a: &(Arc<str>, Arc<str>), r: Record) {
        state.insert(r.get_path(&a.0).expect_string(), r.get_path(&a.1));
    }

    fn finish(state: BTreeMap<Arc<str>, Record>, _a: &(Arc<str>, Arc<str>)) -> Record {
        return Record::from_hash(state);
    }
}
