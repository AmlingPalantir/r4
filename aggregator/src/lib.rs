#[macro_use]
extern crate lazy_static;
extern crate record;
#[macro_use]
extern crate registry;

use record::Record;
use registry::RegistryArgs;
use std::sync::Arc;

registry! {
    AggregatorFe,
    Box<AggregatorState>,
    array,
    concat,
    count,
    count_by,
    distinct_array,
    distinct_concat,
    distinct_count,
    first,
    first_record,
    hash,
    last,
    last_record,
    lexical_max,
    lexical_min,
    lexical_percentile,
    max,
    min,
    percentile,
    record_for_lexical_max,
    record_for_lexical_min,
    record_for_lexical_percentile,
    record_for_max,
    record_for_min,
    record_for_percentile,
    records,
}

pub trait AggregatorState: Send + Sync {
    fn add(&mut self, Record);
    fn finish(self: Box<Self>) -> Record;
    fn box_clone(&self) -> Box<AggregatorState>;
}

pub trait AggregatorFe {
    fn names() -> Vec<&'static str>;
    fn argct() -> usize;
    fn init(&[&str]) -> Box<AggregatorState>;
}

pub trait AggregatorBe {
    type Args: RegistryArgs;
    type State: Clone + Default + Send + Sync;

    fn names() -> Vec<&'static str>;
    fn add(&mut Self::State, &<Self::Args as RegistryArgs>::Val, Record);
    fn finish(Box<Self::State>, &<Self::Args as RegistryArgs>::Val) -> Record;
}

impl<B: AggregatorBe + 'static> AggregatorFe for B {
    fn names() -> Vec<&'static str>{
        return B::names();
    }

    fn argct() -> usize {
        return B::Args::argct();
    }

    fn init(args: &[&str]) -> Box<AggregatorState> {
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

    fn finish(self: Box<Self>) -> Record {
        let a = self.a.clone();
        return B::finish(Box::new(self.s), &a);
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
