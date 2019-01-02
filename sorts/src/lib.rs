#[macro_use]
extern crate lazy_static;
extern crate record;
extern crate rand;
extern crate rand_chacha;
#[macro_use]
extern crate registry;

pub mod bucket;
use self::bucket::KeySortBucket;
use self::bucket::SortBucket;

use record::Record;
use registry::Registrant;
use registry::args::OneStringArgs;
use registry::args::RegistryArgs;
use std::cmp::Reverse;
use std::rc::Rc;
use std::sync::Arc;

pub type BoxedSort = Box<SortInbox>;

registry! {
    BoxedSort,
    lexical,
    numeric,
    shuffle,
}

pub trait SortBe {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn new_bucket(a: &<Self::Args as RegistryArgs>::Val, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket>;
}

pub trait SortInbox: Send + Sync {
    fn new_bucket(&self, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket>;
    fn box_clone(&self) -> BoxedSort;
}

impl Clone for BoxedSort {
    fn clone(&self) -> BoxedSort {
        return self.box_clone();
    }
}

struct SortInboxImpl<B: SortBe> {
    a: Arc<<B::Args as RegistryArgs>::Val>,
}

impl<B: SortBe + 'static> SortInbox for SortInboxImpl<B> {
    fn new_bucket(&self, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket> {
        return B::new_bucket(&self.a, next);
    }

    fn box_clone(&self) -> BoxedSort {
        return Box::new(SortInboxImpl::<B> {
            a: self.a.clone(),
        });
    }
}

pub struct SortRegistrant<B: SortBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: SortBe + 'static> Registrant<BoxedSort> for SortRegistrant<B> {
    type Args = B::Args;

    fn names() -> Vec<&'static str>{
        return B::names();
    }

    fn init2(a: <B::Args as RegistryArgs>::Val) -> BoxedSort {
        return Box::new(SortInboxImpl::<B>{
            a: Arc::new(a),
        });
    }
}

pub trait SortSimpleBe {
    type T: Clone + Ord + 'static;

    fn names() -> Vec<&'static str>;
    fn get(r: Record) -> Self::T;
}

pub struct SortBeFromSimple<B: SortSimpleBe> {
    _x: std::marker::PhantomData<B>,
}

impl<B: SortSimpleBe> SortBe for SortBeFromSimple<B> {
    type Args = OneStringArgs;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn new_bucket(a: &Arc<str>, next: Rc<Fn() -> Box<SortBucket>>) -> Box<SortBucket> {
        let key = a.clone();
        if key.starts_with('-') {
            return KeySortBucket::new(move |r, _i| Reverse(B::get(r.get_path(&key[1..]))), next);
        }
        return KeySortBucket::new(move |r, _i| B::get(r.get_path(&key)), next);
    }
}
