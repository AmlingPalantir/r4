use record::F64SortDishonorProxy;
use record::Record;
use record::RecordTrait;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::lexical_percentile::PercentileArgs;
use super::lexical_percentile::PercentileState;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = PercentileArgs;
    type State = PercentileState<F64SortDishonorProxy>;

    fn names() -> Vec<&'static str> {
        return vec!["recforperc"];
    }

    fn help_msg() -> &'static str {
        return "find the record for a percentile when records are sorted numerically by a value";
    }

    fn add(state: &mut PercentileState<F64SortDishonorProxy>, a: &PercentileArgs, r: Record) {
        let v = r.get_path(&a.key);
        state.add(F64SortDishonorProxy(v.coerce_f64()), r);
    }

    fn finish(state: PercentileState<F64SortDishonorProxy>, a: &PercentileArgs) -> Record {
        return state.finish(a.percentile);
    }
}
