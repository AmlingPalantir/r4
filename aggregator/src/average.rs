use record::Record;
use record::RecordTrait;
use registry::args::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = (f64, f64);

    fn names() -> Vec<&'static str> {
        return vec!["average", "avg"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("key");
    }

    fn help_msg() -> &'static str {
        return "compute average of numeric values";
    }

    fn add(state: &mut (f64, f64), a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        let v = v.coerce_f64();
        state.0 += 1.0;
        state.1 += v;
    }

    fn finish(state: (f64, f64), _a: &Arc<str>) -> Record {
        return Record::from(state.1 / state.0);
    }
}
