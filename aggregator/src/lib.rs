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



#[derive(Clone)]
#[derive(Default)]
struct ZeroArgAggregatorState<S: AggregatorState0> {
    state: S,
}

impl<S: AggregatorState0> AggregatorState0 for ZeroArgAggregatorState<S> {
    fn add(&mut self, r: Record) {
        self.state.add(r);
    }

    fn finish(self) -> Record {
        return self.state.finish();
    }
}

pub trait ZeroArgImplTrait {
    type State;
}

impl<S: AggregatorState0 + Clone + Default + 'static, T: ZeroArgImplTrait<State = S>> Aggregator for T {
    fn argct(&self) -> usize {
        return 1;
    }

    fn state(&self, args: &[String]) -> Box<AggregatorState> {
        assert!(args.is_empty());
        return Box::new(ZeroArgAggregatorState {
            state: S::default(),
        });
    }
}
