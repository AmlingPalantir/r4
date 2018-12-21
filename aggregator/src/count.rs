use AggregatorState0;
use ZeroArgImpl;
use record::FromPrimitive;
use record::Record;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["ct", "count"];
}

#[derive(Default)]
pub struct Impl {
}

impl ZeroArgImpl for Impl {
    type State = State;
}

#[derive(Clone)]
#[derive(Default)]
pub struct State(u32);

impl AggregatorState0 for State {
    fn add(&mut self, _r: Record) {
        self.0 += 1;
    }

    fn finish(self) -> Record {
        return Record::from_primitive(self.0);
    }
}
