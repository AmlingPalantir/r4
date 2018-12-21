use AggregatorBe;
use AggregatorBeState;
use ZeroArgs;
use record::FromPrimitive;
use record::Record;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["ct", "count"];
}

#[derive(Default)]
pub struct Impl {
}

impl AggregatorBe for Impl {
    type Args = ZeroArgs;
    type State = State;
}

#[derive(Clone)]
#[derive(Default)]
pub struct State(u32);

impl AggregatorBeState<()> for State {
    fn add(&mut self, _a: &(), _r: Record) {
        self.0 += 1;
    }

    fn finish(self, _a: &()) -> Record {
        return Record::from_primitive(self.0);
    }
}
