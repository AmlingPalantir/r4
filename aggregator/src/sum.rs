use AggregatorBe;
use record::Record;
use registry::OneStringArgs;
use std::sync::Arc;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = f64;

    fn names() -> Vec<&'static str> {
        return vec!["sum"];
    }

    fn add(state: &mut f64, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        *state += v.coerce_f64();
    }

    fn finish(state: Box<f64>, _a: &Arc<str>) -> Record {
        return Record::from(*state);
    }
}
