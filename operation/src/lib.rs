extern crate record;
extern crate stream;

#[macro_use]
extern crate registry;

registry! {
    Operation:
    bg,
    test,
}

use std::collections::VecDeque;
use stream::Stream;

pub trait Operation {
    fn validate(&self, &mut VecDeque<String>) -> StreamWrapper;
}

pub struct StreamWrapper(Box<Fn(Stream) -> Stream>);

impl StreamWrapper {
    pub fn new<F: Fn(Stream) -> Stream + 'static>(f: F) -> Self {
        return StreamWrapper(Box::new(f));
    }

    pub fn wrap(&self, os: Stream) -> Stream {
        return self.0(os);
    }
}
