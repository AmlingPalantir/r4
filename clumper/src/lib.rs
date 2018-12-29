#[macro_use]
extern crate lazy_static;
extern crate record;
#[macro_use]
extern crate registry;
extern crate stream;

use record::Record;
use registry::RegistryArgs;
use std::rc::Rc;
use std::sync::Arc;
use stream::Stream;

registry! {
    ClumperFe,
    Box<ClumperWrapper>,
    key,
    round_robin,
    window,
}

pub trait ClumperFe {
    fn names() -> Vec<&'static str>;
    fn argct() -> usize;
    fn init(args: &[&str]) -> Box<ClumperWrapper>;
}

pub trait ClumperWrapper: Send + Sync {
    fn stream(&self, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream;
    fn box_clone(&self) -> Box<ClumperWrapper>;
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
        return Box::new(ClumperWrapperImpl::<B>(Arc::from(B::Args::parse(args))));
    }
}

struct ClumperWrapperImpl<B: ClumperBe>(Arc<<<B as ClumperBe>::Args as RegistryArgs>::Val>);

impl<B: ClumperBe + 'static> ClumperWrapper for ClumperWrapperImpl<B> {
    fn stream(&self, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        return B::stream(&self.0, bsw);
    }

    fn box_clone(&self) -> Box<ClumperWrapper> {
        return Box::new(ClumperWrapperImpl::<B>(self.0.clone()));
    }
}

impl Clone for Box<ClumperWrapper> {
    fn clone(&self) -> Box<ClumperWrapper> {
        return self.box_clone();
    }
}

pub fn stream<F: Fn(Vec<(Arc<str>, Record)>) -> Stream + 'static>(cws: &Vec<Box<ClumperWrapper>>, f: F) -> Stream {
    let mut bsw: Rc<Fn(Vec<(Arc<str>, Record)>) -> Stream> = Rc::new(f);

    bsw = cws.iter().rev().fold(bsw, |bsw, cw| {
        let cw = cw.clone();
        return Rc::new(move |bucket_outer| {
            let bucket_outer = bucket_outer.clone();
            let bsw = bsw.clone();
            return cw.stream(Box::new(move |bucket_inner| {
                let mut bucket = bucket_outer.clone();
                bucket.extend(bucket_inner);
                return bsw(bucket);
            }));
        });
    });

    return bsw(vec![]);
}
