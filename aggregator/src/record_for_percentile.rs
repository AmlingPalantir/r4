use AggregatorBe;
use record::Record;
use record::RecordTrait;
use record::float::F64SortDishonorProxy;
use std::sync::Arc;
use super::lexical_percentile::PercentileArgs;
use super::lexical_percentile::PercentileState;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = PercentileArgs;
    type State = PercentileState<F64SortDishonorProxy>;

    fn names() -> Vec<&'static str> {
        return vec!["recforperc"];
    }

    fn add(state: &mut PercentileState<F64SortDishonorProxy>, a: &(f64, Arc<str>), r: Record) {
        let v = r.get_path(&a.1);
        state.add(F64SortDishonorProxy(v.coerce_f64()), r);
    }

    fn finish(state: Box<PercentileState<F64SortDishonorProxy>>, a: &(f64, Arc<str>)) -> Record {
        return state.finish(a.0);
    }
}
