extern crate record;
extern crate stream;

#[macro_use]
extern crate registry;

registry! {
    Operation:
    test,
}

use stream::Stream;

pub trait Operation {
    fn configure(&mut self, Vec<String>) -> Vec<String>;
    fn validate(&self) -> Box<StreamWrapper>;
}

pub trait StreamWrapper {
    fn wrap(&self, Box<Stream>) -> Box<Stream>;
}

struct ClosureStreamWrapper(Box<Fn(Box<Stream>) -> Box<Stream>>);

impl ClosureStreamWrapper {
    fn new<F: Fn(Box<Stream>) -> Box<Stream> + 'static>(f: F) -> Box<StreamWrapper> {
        return Box::new(ClosureStreamWrapper(Box::new(f)));
    }
}

impl StreamWrapper for ClosureStreamWrapper {
    fn wrap(&self, os: Box<Stream>) -> Box<Stream> {
        return self.0(os);
    }
}
