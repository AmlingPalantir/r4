use Aggregator;
use AggregatorState0;
use AggregatorState;
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
pub struct State(Vec<Record>);

impl AggregatorState0 for State {
    fn add(&mut self, r: Record) {
        self.0.push(r);
    }

    fn finish(self) -> Record {
        return Record::from_vec(self.0);
    }
}
