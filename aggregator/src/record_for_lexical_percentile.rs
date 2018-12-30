use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::lexical_percentile::PercentileArgs;
use super::lexical_percentile::PercentileState;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = PercentileArgs;
    type State = PercentileState<Arc<str>>;

    fn names() -> Vec<&'static str> {
        return vec!["recforlperc"];
    }

    fn add(state: &mut PercentileState<Arc<str>>, a: &(f64, Arc<str>), r: Record) {
        let v = r.get_path(&a.1);
        state.add(v.expect_string(), r);
    }

    fn finish(state: Box<PercentileState<Arc<str>>>, a: &(f64, Arc<str>)) -> Record {
        return state.finish(a.0);
    }
}
