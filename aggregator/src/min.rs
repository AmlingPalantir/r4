use AggregatorBe;
use record::Record;
use registry::OneStringArgs;
use std::sync::Arc;
use super::lexical_max::MaxState;
use super::lexical_min::ReverseOrd;
use super::max::F64DishonorProxy;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = MaxState<ReverseOrd<F64DishonorProxy>>;

    fn names() -> Vec<&'static str> {
        return vec!["min"];
    }

    fn add(state: &mut MaxState<ReverseOrd<F64DishonorProxy>>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(ReverseOrd(F64DishonorProxy(v.coerce_f64())), v);
    }

    fn finish(state: Box<MaxState<ReverseOrd<F64DishonorProxy>>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
