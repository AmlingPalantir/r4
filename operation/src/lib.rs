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
