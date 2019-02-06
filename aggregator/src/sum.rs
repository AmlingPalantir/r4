use misc::Either;
use record::Record;
use record::RecordTrait;
use registry::args::OneStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

#[derive(Clone)]
pub(crate) struct State(Either<i64, f64>);

impl Default for State {
    fn default() -> Self {
        return State(Either::Left(0));
    }
}

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = State;

    fn names() -> Vec<&'static str> {
        return vec!["sum"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("key");
    }

    fn help_msg() -> &'static str {
        return "compute sum of numeric values";
    }

    fn add(state: &mut State, a: &Arc<str>, r: Record) {
        let n1 = r.get_path(a).coerce_num();
        let n2 = state.0.clone();

        if let Either::Left(i1) = n1 {
            if let Either::Left(i2) = n2 {
                *state = State(Either::Left(i1 + i2));
                return;
            }
        }

        let f1 = n1.map_left(|i| i as f64).join();
        let f2 = n2.map_left(|i| i as f64).join();

        *state = State(Either::Right(f1 + f2));
    }

    fn finish(state: State, _a: &Arc<str>) -> Record {
        return state.0.map_left(Record::from).map_right(Record::from).join();
    }
}
