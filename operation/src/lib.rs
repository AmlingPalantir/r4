extern crate record;
extern crate stream;

#[macro_use]
extern crate registry;

registry! {
    Operation:
    test,
}

use std::collections::VecDeque;
use stream::Stream;

pub trait Operation {
    fn validate(&self, &mut VecDeque<String>) -> Box<StreamWrapper>;
}

pub trait StreamWrapper {
    fn wrap(&self, Stream) -> Stream;
}

struct ClosureStreamWrapper(Box<Fn(Stream) -> Stream>);

impl ClosureStreamWrapper {
    fn new<F: Fn(Stream) -> Stream + 'static>(f: F) -> Box<StreamWrapper> {
        return Box::new(ClosureStreamWrapper(Box::new(f)));
    }
}

impl StreamWrapper for ClosureStreamWrapper {
    fn wrap(&self, os: Stream) -> Stream {
        return self.0(os);
    }
}
