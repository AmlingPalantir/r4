use record::F64SortDishonorProxy;
use record::Record;
use record::RecordTrait;
use registry::args::OneKeyRegistryArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::lexical_max::MaxState;
use super::lexical_min::ReverseOrd;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneKeyRegistryArgs;
    type State = MaxState<ReverseOrd<F64SortDishonorProxy>>;

    fn names() -> Vec<&'static str> {
        return vec!["min"];
    }

    fn help_msg() -> &'static str {
        return "track the numerically minimal value";
    }

    fn add(state: &mut MaxState<ReverseOrd<F64SortDishonorProxy>>, a: &OneKeyRegistryArgs, r: Record) {
        let v = r.get_path(&a.key);
        state.add(ReverseOrd(F64SortDishonorProxy(v.coerce_f64())), v);
    }

    fn finish(state: MaxState<ReverseOrd<F64SortDishonorProxy>>, _a: &OneKeyRegistryArgs) -> Record {
        return state.finish();
    }
}
