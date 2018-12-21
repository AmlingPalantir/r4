extern crate record;
#[macro_use]
extern crate registry;

use record::Record;

registry! {
    Aggregator:
    count,
    records,
}

pub trait Aggregator {
    fn argct(&self) -> usize;
    fn state(&self, args: &[String]) -> Box<AggregatorState>;
}

pub trait AggregatorState0 {
    fn add(&mut self, Record);
    fn finish(self) -> Record;
}

pub trait AggregatorState: AggregatorState0 {
    fn box_clone(&self) -> Box<AggregatorState>;
}

impl<S: AggregatorState0 + Clone + 'static> AggregatorState for S {
    fn box_clone(&self) -> Box<AggregatorState> {
        return Box::new(self.clone());
    }
}

impl Clone for Box<AggregatorState> {
    fn clone(&self) -> Self {
        return self.box_clone();
    }
}
