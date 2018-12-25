use AggregatorBe;
use record::Record;
use registry::OneStringArgs;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::sync::Arc;
use super::lexical_max::MaxState;

pub struct Impl();

#[derive(Clone)]
pub struct F64DishonorProxy(pub f64);

impl PartialEq for F64DishonorProxy {
    fn eq(&self, other: &F64DishonorProxy) -> bool {
        assert!(!self.0.is_nan());
        assert!(!other.0.is_nan());
        return self.0 == other.0;
    }
}

impl PartialOrd for F64DishonorProxy {
    fn partial_cmp(&self, other: &F64DishonorProxy) -> Option<Ordering> {
        assert!(!self.0.is_nan());
        assert!(!other.0.is_nan());
        return self.0.partial_cmp(&other.0);
    }
}

impl Eq for F64DishonorProxy {
}

impl Ord for F64DishonorProxy {
    fn cmp(&self, other: &F64DishonorProxy) -> Ordering {
        assert!(!self.0.is_nan());
        assert!(!other.0.is_nan());
        return self.0.partial_cmp(&other.0).unwrap();
    }
}

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = MaxState<F64DishonorProxy>;

    fn names() -> Vec<&'static str> {
        return vec!["max"];
    }

    fn add(state: &mut MaxState<F64DishonorProxy>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(F64DishonorProxy(v.coerce_f64()), v);
    }

    fn finish(state: Box<MaxState<F64DishonorProxy>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
