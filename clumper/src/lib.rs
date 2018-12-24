#[macro_use]
extern crate lazy_static;
extern crate record;
#[macro_use]
extern crate registry;
extern crate stream;

use record::Record;
use registry::RegistryArgs;
use std::sync::Arc;
use stream::Stream;

registry! {
    ClumperFe,
    Box<ClumperWrapper>,
    key,
}

pub trait ClumperFe {
    fn names() -> Vec<&'static str>;
    fn argct() -> usize;
    fn init(args: &[&str]) -> Box<ClumperWrapper>;
}

pub trait ClumperWrapper: Send + Sync {
    fn stream(&self, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream;
}

pub trait ClumperBe {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn stream(&<Self::Args as RegistryArgs>::Val, Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream;
}

impl<B: ClumperBe + 'static> ClumperFe for B {
    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn argct() -> usize {
        return B::Args::argct();
    }

    fn init(args: &[&str]) -> Box<ClumperWrapper> {
        return Box::new(ClumperWrapperImpl::<B> {
            a: Arc::from(B::Args::parse(args)),
        });
    }
}

struct ClumperWrapperImpl<B: ClumperBe> {
    a: Arc<<<B as ClumperBe>::Args as RegistryArgs>::Val>,
}

impl<B: ClumperBe> ClumperWrapper for ClumperWrapperImpl<B> {
    fn stream(&self, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        return B::stream(&self.a, bsw);
    }
}
