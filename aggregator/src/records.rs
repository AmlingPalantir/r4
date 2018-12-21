use AggregatorState0;
use ZeroArgImplTrait;
use record::Record;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["ct", "count"];
}

#[derive(Default)]
pub struct Impl {
}

impl ZeroArgImplTrait for Impl {
    type State = State;
}

#[derive(Clone)]
#[derive(Default)]
pub struct State(Vec<Record>);

impl AggregatorState0 for State {
    fn add(&mut self, r: Record) {
        self.0.push(r);
    }

    fn finish(self) -> Record {
        return Record::from_vec(self.0);
    }
}
