use record::F64SortDishonorProxy;
use record::Record;
use record::RecordTrait;
use registry::args::OneKeyRegistryArgs;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::lexical_max::MaxState;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneKeyRegistryArgs;
    type State = MaxState<F64SortDishonorProxy>;

    fn names() -> Vec<&'static str> {
        return vec!["recformax"];
    }

    fn help_msg() -> &'static str {
        return "track the record for the numerically maximal value";
    }

    fn add(state: &mut MaxState<F64SortDishonorProxy>, a: &OneKeyRegistryArgs, r: Record) {
        let v = r.get_path(&a.key);
        state.add(F64SortDishonorProxy(v.coerce_f64()), r);
    }

    fn finish(state: MaxState<F64SortDishonorProxy>, _a: &OneKeyRegistryArgs) -> Record {
        return state.finish();
    }
}
