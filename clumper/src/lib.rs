#[macro_use]
extern crate lazy_static;
extern crate record;
#[macro_use]
extern crate registry;
extern crate stream;

use record::Record;
use registry::Registrant;
use registry::args::RegistryArgs;
use std::rc::Rc;
use std::sync::Arc;
use stream::Stream;

pub type BoxedClumper = Box<ClumperInbox>;

registry! {
    BoxedClumper,
    key,
    round_robin,
    window,
}

pub trait ClumperBe {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn stream(a: &<Self::Args as RegistryArgs>::Val, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream;
}

pub trait ClumperInbox: Send + Sync {
    fn stream(&self, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream;
    fn box_clone(&self) -> BoxedClumper;
}

impl Clone for BoxedClumper {
    fn clone(&self) -> BoxedClumper {
        return self.box_clone();
    }
}

struct ClumperInboxImpl<B: ClumperBe> {
    a: Arc<<B::Args as RegistryArgs>::Val>,
}

impl<B: ClumperBe + 'static> ClumperInbox for ClumperInboxImpl<B> {
    fn stream(&self, bsw: Box<Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        return B::stream(&self.a, bsw);
    }

    fn box_clone(&self) -> BoxedClumper {
        return Box::new(ClumperInboxImpl::<B>{
            a: self.a.clone(),
        });
    }
}

pub struct ClumperRegistrant<B: ClumperBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: ClumperBe + 'static> Registrant<BoxedClumper> for ClumperRegistrant<B> {
    type Args = B::Args;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn init2(a: <B::Args as RegistryArgs>::Val) -> BoxedClumper {
        return Box::new(ClumperInboxImpl::<B>{
            a: Arc::new(a),
        });
    }
}

pub fn stream<F: Fn(Vec<(Arc<str>, Record)>) -> Stream + 'static>(cws: &Vec<BoxedClumper>, f: F) -> Stream {
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
