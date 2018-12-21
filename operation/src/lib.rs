extern crate bgop;
extern crate opts;
extern crate record;
extern crate stream;

#[macro_use]
extern crate registry;

registry! {
    Operation:
    bg,
    test,
}

use stream::Stream;

pub trait Operation {
    fn validate(&self, &mut Vec<String>) -> StreamWrapper;
}

pub struct StreamWrapper(Box<Fn(Stream) -> Stream + Send + Sync>);

impl StreamWrapper {
    pub fn new<F: Fn(Stream) -> Stream + 'static + Send + Sync>(f: F) -> Self {
        return StreamWrapper(Box::new(f));
    }

    pub fn wrap(&self, os: Stream) -> Stream {
        return self.0(os);
    }
}
