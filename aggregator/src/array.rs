use AggregatorState0;
use ZeroArgImpl;
use record::Record;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["arr", "array"];
}

#[derive(Default)]
pub struct Impl {
}

impl OneKeyImpl for Impl {
    type State = State;
}

#[derive(Clone)]
#[derive(Default)]
pub struct State(Vec<Record>);

impl OneKeyAggregatorState for State {
    fn add(&mut self, v: Record, _r: Record) {
        self.0.push(v);
    }

    fn finish(self) -> Record {
        return Record::from_vec(self.0);
    }
}
