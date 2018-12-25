use AggregatorBe;
use record::Record;
use registry::OneStringArgs;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = HashMap<Arc<str>, i64>;

    fn names() -> Vec<&'static str> {
        return vec!["countby", "ctby", "cb"];
    }

    fn add(state: &mut HashMap<Arc<str>, i64>, a: &Arc<str>, r: Record) {
        *state.entry(r.get_path(a).expect_string()).or_insert(0) += 1;
    }

    fn finish(state: Box<HashMap<Arc<str>, i64>>, _a: &Arc<str>) -> Record {
        return Record::from_hash(state.into_iter().map(|(v, ct)| (v, Record::from_i64(ct))).collect());
    }
}
