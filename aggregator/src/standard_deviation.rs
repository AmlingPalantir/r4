use record::Record;
use record::RecordTrait;
use registry::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = (f64, f64, f64);

    fn names() -> Vec<&'static str> {
        return vec!["stddev", "sd"];
    }

    fn add(state: &mut (f64, f64, f64), a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        let v = v.coerce_f64();
        state.0 += 1.0;
        state.1 += v;
        state.2 += v * v;
    }

    fn finish(state: Box<(f64, f64, f64)>, _a: &Arc<str>) -> Record {
        return Record::from(((state.2 / state.0) - (state.1 / state.0).powi(2)).sqrt());
    }
}
