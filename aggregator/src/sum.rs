use record::Record;
use record::RecordTrait;
use registry::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
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
