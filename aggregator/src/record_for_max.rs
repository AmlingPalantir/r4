use AggregatorBe;
use record::Record;
use registry::OneStringArgs;
use std::sync::Arc;
use super::lexical_max::MaxState;
use super::max::F64DishonorProxy;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = MaxState<F64DishonorProxy>;

    fn names() -> Vec<&'static str> {
        return vec!["recformax"];
    }

    fn add(state: &mut MaxState<F64DishonorProxy>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(F64DishonorProxy(v.coerce_f64()), r);
    }

    fn finish(state: Box<MaxState<F64DishonorProxy>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
