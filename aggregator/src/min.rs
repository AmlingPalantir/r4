use record::F64SortDishonorProxy;
use record::Record;
use record::RecordTrait;
use registry::args::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::lexical_max::MaxState;
use super::lexical_min::ReverseOrd;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = MaxState<ReverseOrd<F64SortDishonorProxy>>;

    fn names() -> Vec<&'static str> {
        return vec!["min"];
    }

    fn add(state: &mut MaxState<ReverseOrd<F64SortDishonorProxy>>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(ReverseOrd(F64SortDishonorProxy(v.coerce_f64())), v);
    }

    fn finish(state: Box<MaxState<ReverseOrd<F64SortDishonorProxy>>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
