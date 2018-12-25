use AggregatorBe;
use record::Record;
use std::sync::Arc;
use super::lexical_percentile::PercentileArgs;
use super::lexical_percentile::PercentileState;
use super::max::F64DishonorProxy;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = PercentileArgs;
    type State = PercentileState<F64DishonorProxy>;

    fn names() -> Vec<&'static str> {
        return vec!["perc"];
    }

    fn add(state: &mut PercentileState<F64DishonorProxy>, a: &(f64, Arc<str>), r: Record) {
        let v = r.get_path(&a.1);
        state.add(F64DishonorProxy(v.coerce_f64()), v);
    }

    fn finish(state: Box<PercentileState<F64DishonorProxy>>, a: &(f64, Arc<str>)) -> Record {
        return state.finish(a.0);
    }
}
