#[macro_use]
extern crate lazy_static;
extern crate record;
#[macro_use]
extern crate registry;

use record::Record;
use registry::OneStringArgs;
use registry::RegistryArgs;
use std::cmp::Ordering;
use std::sync::Arc;

registry! {
    SortFe,
    Box<SortState>,
    lexical,
    numeric,
}

pub trait SortState: Send + Sync {
    fn cmp(&self, r1: &Record, r2: &Record) -> Ordering;
    fn box_clone(&self) -> Box<SortState>;
}

pub trait SortFe {
    fn names() -> Vec<&'static str>;
    fn argct() -> usize;
    fn init(args: &[&str]) -> Box<SortState>;
}

pub trait SortBe {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn cmp(a: &<Self::Args as RegistryArgs>::Val, r1: &Record, r2: &Record) -> Ordering;
}

impl<B: SortBe + 'static> SortFe for B {
    fn names() -> Vec<&'static str>{
        return B::names();
    }

    fn argct() -> usize {
        return B::Args::argct();
    }

    fn init(args: &[&str]) -> Box<SortState> {
        return Box::new(SortStateImpl::<B> {
            a: Arc::from(B::Args::parse(args)),
        });
    }
}

struct SortStateImpl<B: SortBe> {
    a: Arc<<B::Args as RegistryArgs>::Val>,
}

impl<B: SortBe + 'static> SortState for SortStateImpl<B> {
    fn cmp(&self, r1: &Record, r2: &Record) -> Ordering {
        return B::cmp(&self.a, r1, r2);
    }

    fn box_clone(&self) -> Box<SortState> {
        return Box::new(SortStateImpl::<B> {
            a: self.a.clone(),
        });
    }
}

impl Clone for Box<SortState> {
    fn clone(&self) -> Box<SortState> {
        return self.box_clone();
    }
}

pub trait SortSimpleBe {
    type T: Ord;

    fn names() -> Vec<&'static str>;
    fn get(r: Record) -> Self::T;
}

pub struct SortSimpleBeImpl<B: SortSimpleBe> {
    _x: std::marker::PhantomData<B>,
}

impl<B: SortSimpleBe> SortBe for SortSimpleBeImpl<B> {
    type Args = OneStringArgs;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn cmp(a: &Arc<str>, r1: &Record, r2: &Record) -> Ordering {
        let mut reverse = false;
        let mut key = &a as &str;
        if key.starts_with('-') {
            reverse = true;
            key = &key[1..];
        }

        let v1 = B::get(r1.get_path(key));
        let v2 = B::get(r2.get_path(key));
        let mut r = v1.cmp(&v2);
        if reverse {
            r = r.reverse();
        }
        return r;
    }
}
