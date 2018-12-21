extern crate record;
#[macro_use]
extern crate registry;

use record::Record;

registry! {
    Aggregator:
    count,
}

pub trait Aggregator0 {
    fn add(&mut self, Record);
    fn finish(self) -> Record;
}

pub trait Aggregator: Aggregator0 {
    fn box_clone(&self) -> Box<Aggregator>;
}

impl<T: Aggregator0 + Clone + 'static> Aggregator for T {
    fn box_clone(&self) -> Box<Aggregator> {
        return Box::new(self.clone());
    }
}

impl Clone for Box<Aggregator> {
    fn clone(&self) -> Self {
        return self.box_clone();
    }
}
