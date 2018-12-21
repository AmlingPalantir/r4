extern crate record;
#[macro_use]
extern crate registry;

use record::Record;
use std::sync::Arc;

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
struct ZeroArgAggregatorStateHolder<S: AggregatorState0> {
    state: S,
}

impl<S: AggregatorState0> AggregatorState0 for ZeroArgAggregatorStateHolder<S> {
    fn add(&mut self, r: Record) {
        self.state.add(r);
    }

    fn finish(self) -> Record {
        return self.state.finish();
    }
}

pub trait ZeroArgImpl {
    type State;
}

impl<S: AggregatorState0 + Clone + Default + 'static, T: ZeroArgImpl<State = S>> Aggregator for T {
    fn argct(&self) -> usize {
        return 1;
    }

    fn state(&self, args: &[String]) -> Box<AggregatorState> {
        assert!(args.is_empty());
        return Box::new(ZeroArgAggregatorStateHolder {
            state: S::default(),
        });
    }
}



pub trait OneKeyAggregatorState {
    fn add(&mut self, Record, Record);
    fn finish(self) -> Record;
}

#[derive(Clone)]
#[derive(Default)]
struct OneKeyAggregatorStateHolder<S: OneKeyAggregatorState> {
    path: Arc<str>,
    state: S,
}

impl<S: OneKeyAggregatorState> AggregatorState0 for OneKeyAggregatorStateHolder<S> {
    fn add(&mut self, r: Record) {
        self.state.add(r.get_path(self.path), r);
    }

    fn finish(self) -> Record {
        return self.state.finish();
    }
}

pub trait OneKeyImpl {
    type State;
}

impl<S: OneKeyAggregatorState + Clone + Default + 'static, T: OneKeyImpl<State = S>> Aggregator for T {
    fn argct(&self) -> usize {
        return 1;
    }

    fn state(&self, args: &[String]) -> Box<AggregatorState> {
        let path = Arc::from(args[0]);
        return Box::new(OneKeyAggregatorStateHolder {
            path: path,
            state: S::default(),
        });
    }
}
