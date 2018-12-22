extern crate record;
#[macro_use]
extern crate registry;

use record::Record;
use registry::RegistryArgs;
use std::sync::Arc;

registry! {
    AggregatorFe:
    array,
    count,
    records,
}

pub trait AggregatorFe {
    fn argct(&self) -> usize;
    fn state(&self, args: &[&str]) -> Box<AggregatorState>;
}

pub trait AggregatorState: Send + Sync {
    fn add(&mut self, Record);
    fn finish(self) -> Record;
    fn box_clone(&self) -> Box<AggregatorState>;
}

pub trait AggregatorBe {
    type Args: RegistryArgs;
    type State: Clone + Default + Send + Sync;

    fn add(&mut Self::State, &<Self::Args as RegistryArgs>::Val, Record);
    fn finish(Self::State, &<Self::Args as RegistryArgs>::Val) -> Record;
}

impl<B: AggregatorBe + 'static> AggregatorFe for B {
    fn argct(&self) -> usize {
        return B::Args::argct();
    }

    fn state(&self, args: &[&str]) -> Box<AggregatorState> {
        return Box::new(AggregatorStateImpl::<B> {
            a: Arc::from(B::Args::parse(args)),
            s: B::State::default(),
        });
    }
}

struct AggregatorStateImpl<B: AggregatorBe> {
    a: Arc<<<B as AggregatorBe>::Args as RegistryArgs>::Val>,
    s: B::State,
}

impl<B: AggregatorBe + 'static> AggregatorState for AggregatorStateImpl<B> {
    fn add(&mut self, r: Record) {
        B::add(&mut self.s, &self.a, r);
    }

    fn finish(self) -> Record {
        return B::finish(self.s, &self.a);
    }

    fn box_clone(&self) -> Box<AggregatorState> {
        return Box::new(AggregatorStateImpl::<B> {
            a: self.a.clone(),
            s: self.s.clone(),
        });
    }
}

impl Clone for Box<AggregatorState> {
    fn clone(&self) -> Box<AggregatorState> {
        return self.box_clone();
    }
}
