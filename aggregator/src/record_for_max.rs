use record::F64SortDishonorProxy;
use record::Record;
use record::RecordTrait;
use registry::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::lexical_max::MaxState;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = MaxState<F64SortDishonorProxy>;

    fn names() -> Vec<&'static str> {
        return vec!["recformax"];
    }

    fn add(state: &mut MaxState<F64SortDishonorProxy>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(F64SortDishonorProxy(v.coerce_f64()), r);
    }

    fn finish(state: Box<MaxState<F64SortDishonorProxy>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
