use Aggregator;
use AggregatorState0;
use AggregatorState;
use record::FromPrimitive;
use record::Record;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["ct", "count"];
}

#[derive(Default)]
pub struct Impl {
}

impl Aggregator for Impl {
    fn argct(&self) -> usize {
        return 0;
    }

    fn state(&self, args: &[String]) -> Box<AggregatorState> {
        assert!(args.is_empty());
        return Box::new(State::default());
    }
}

#[derive(Clone)]
#[derive(Default)]
struct State(u32);

impl AggregatorState0 for State {
    fn add(&mut self, _r: Record) {
        self.0 += 1;
    }

    fn finish(self) -> Record {
        return Record::from_primitive(self.0);
    }
}
