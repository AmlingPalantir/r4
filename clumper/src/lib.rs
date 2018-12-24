extern crate record;
#[macro_use]
extern crate registry;
extern crate stream;

use record::Record;
use registry::RegistryArgs;
use std::sync::Arc;
use stream::Stream;

registry! {
    ClumperFe:
    key,
}

pub trait ClumperFe {
    fn argct(&self) -> usize;
    fn wrapper(&self, args: &[&str]) -> Box<ClumperWrapper>;
}

pub trait ClumperWrapper: Send + Sync {
    fn stream(&self, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream;
}

pub trait ClumperBe {
    type Args: RegistryArgs;

    fn stream(&<Self::Args as RegistryArgs>::Val, Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream;
}

impl<B: ClumperBe + 'static> ClumperFe for B {
    fn argct(&self) -> usize {
        return B::Args::argct();
    }

    fn wrapper(&self, args: &[&str]) -> Box<ClumperWrapper> {
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
