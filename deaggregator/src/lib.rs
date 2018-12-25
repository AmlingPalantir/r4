#[macro_use]
extern crate lazy_static;
extern crate record;
#[macro_use]
extern crate registry;

use record::Record;
use registry::RegistryArgs;
use std::sync::Arc;

registry! {
    DeaggregatorFe,
    Box<DeaggregatorState>,
    split,
    unarray,
    unhash,
}

pub trait DeaggregatorFe {
    fn names() -> Vec<&'static str>;
    fn argct() -> usize;
    fn init(args: &[&str]) -> Box<DeaggregatorState>;
}

pub trait DeaggregatorState: Send + Sync {
    fn deaggregate(&self, Record) -> Vec<Vec<(Arc<str>, Record)>>;
    fn box_clone(&self) -> Box<DeaggregatorState>;
}

pub trait DeaggregatorBe {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn deaggregate(&<Self::Args as RegistryArgs>::Val, Record) -> Vec<Vec<(Arc<str>, Record)>>;
}

impl<B: DeaggregatorBe + 'static> DeaggregatorFe for B {
    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn argct() -> usize {
        return B::Args::argct();
    }

    fn init(args: &[&str]) -> Box<DeaggregatorState> {
        return Box::new(DeaggregatorStateImpl::<B>(Arc::from(B::Args::parse(args))));
    }
}

struct DeaggregatorStateImpl<B: DeaggregatorBe>(Arc<<<B as DeaggregatorBe>::Args as RegistryArgs>::Val>);

impl<B: DeaggregatorBe + 'static> DeaggregatorState for DeaggregatorStateImpl<B> {
    fn deaggregate(&self, r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        return B::deaggregate(&self.0, r);
    }

    fn box_clone(&self) -> Box<DeaggregatorState> {
        return Box::new(DeaggregatorStateImpl::<B>(self.0.clone()));
    }
}

impl Clone for Box<DeaggregatorState> {
    fn clone(&self) -> Box<DeaggregatorState> {
        return self.box_clone();
    }
}
