extern crate record;
extern crate stream;

pub mod test;

use stream::Stream;

pub trait Operation {
    fn wrap(&self, Box<Stream>) -> Box<Stream>;
}
