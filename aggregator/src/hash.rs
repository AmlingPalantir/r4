use AggregatorBe;
use record::Record;
use record::RecordTrait;
use registry::TwoStringArgs;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = TwoStringArgs;
    type State = BTreeMap<Arc<str>, Record>;

    fn names() -> Vec<&'static str> {
        return vec!["hash"];
    }

    fn add(state: &mut BTreeMap<Arc<str>, Record>, a: &(Arc<str>, Arc<str>), r: Record) {
        state.insert(r.get_path(&a.0).expect_string(), r.get_path(&a.1));
    }

    fn finish(state: Box<BTreeMap<Arc<str>, Record>>, _a: &(Arc<str>, Arc<str>)) -> Record {
        return Record::from_hash(*state);
    }
}
