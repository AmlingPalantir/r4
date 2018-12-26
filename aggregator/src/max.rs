use AggregatorBe;
use record::Record;
use record::float::F64SortDishonorProxy;
use registry::OneStringArgs;
use std::sync::Arc;
use super::lexical_max::MaxState;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = MaxState<F64SortDishonorProxy>;

    fn names() -> Vec<&'static str> {
        return vec!["max"];
    }

    fn add(state: &mut MaxState<F64SortDishonorProxy>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(F64SortDishonorProxy(v.coerce_f64()), v);
    }

    fn finish(state: Box<MaxState<F64SortDishonorProxy>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
