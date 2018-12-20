extern crate record;
extern crate stream;

pub mod test;

pub fn find_operation(name: &str) -> Box<Operation> {
    if name == test::name() {
        return test::new();
    }
    panic!();
}

use stream::Stream;

pub trait Operation {
    fn configure(&mut self, Vec<String>) -> Vec<String>;
    fn validate(&self) -> Box<StreamWrapper>;
}

pub trait StreamWrapper {
    fn wrap(&self, Box<Stream>) -> Box<Stream>;
}
