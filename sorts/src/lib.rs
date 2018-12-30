#[macro_use]
extern crate lazy_static;
extern crate record;
extern crate rand;
extern crate rand_chacha;
#[macro_use]
extern crate registry;

#[cfg(test)]
mod tests;

use record::Record;
use registry::OneStringArgs;
use registry::Registrant;
use registry::RegistryArgs;
use std::sync::Arc;

registry! {
    Box<SortInbox>,
    lexical,
    numeric,
    shuffle,
}

trait SortBe {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn sort(a: &<Self::Args as RegistryArgs>::Val, rs: &mut [Record]);
    fn sort_aux<'a>(a: &<Self::Args as RegistryArgs>::Val, ct: usize, f: Box<Fn(usize) -> &'a Record + 'a>) -> Vec<usize>;
}

pub trait SortInbox: Send + Sync {
    fn sort(&self, rs: &mut [Record]);
    fn sort_aux<'a>(&self, ct: usize, f: Box<Fn(usize) -> &'a Record + 'a>) -> Vec<usize>;
    fn box_clone(&self) -> Box<SortInbox>;
}

impl Clone for Box<SortInbox> {
    fn clone(&self) -> Box<SortInbox> {
        return self.box_clone();
    }
}

struct SortInboxImpl<B: SortBe> {
    a: Arc<<B::Args as RegistryArgs>::Val>,
}

impl<B: SortBe + 'static> SortInbox for SortInboxImpl<B> {
    fn sort(&self, rs: &mut [Record]) {
        return B::sort(&self.a, rs);
    }

    fn sort_aux<'a>(&self, ct: usize, f: Box<Fn(usize) -> &'a Record + 'a>) -> Vec<usize> {
        return B::sort_aux(&self.a, ct, f);
    }

    fn box_clone(&self) -> Box<SortInbox> {
        return Box::new(SortInboxImpl::<B> {
            a: self.a.clone(),
        });
    }
}

struct SortRegistrant<B: SortBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: SortBe + 'static> Registrant<Box<SortInbox>> for SortRegistrant<B> {
    type Args = B::Args;

    fn names() -> Vec<&'static str>{
        return B::names();
    }

    fn init2(a: <B::Args as RegistryArgs>::Val) -> Box<SortInbox> {
        return Box::new(SortInboxImpl::<B>{
            a: Arc::new(a),
        });
    }
}

pub trait SortSimpleBe {
    type T: Ord;

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

    fn sort(a: &Arc<str>, rs: &mut [Record]) {
        let idxs = SortBeFromSimple::<B>::sort_aux(a, rs.len(), Box::new(|i| &rs[i]));
        reorder(rs, &idxs);
    }

    fn sort_aux<'a>(a: &Arc<str>, ct: usize, f: Box<Fn(usize) -> &'a Record + 'a>) -> Vec<usize> {
        let mut reverse = false;
        let mut key = &a as &str;
        if key.starts_with('-') {
            reverse = true;
            key = &key[1..];
        }

        let mut pairs: Vec<_> = (0..ct).map(|i| (i, B::get(f(i).get_path(key)))).collect();
        pairs.sort_by(|(_, t1), (_, t2)| t1.cmp(t2));
        if reverse {
            pairs.reverse();
        }
        return pairs.into_iter().map(|(i, _)| i).collect();
    }
}

pub fn reorder<T>(ts: &mut [T], idxs: &[usize]) {
    let ct = ts.len();
    assert_eq!(idxs.len(), ct);

    // Make our own "fw" copy of idxs and its reverse.  Also sanity check
    // everything.
    let mut fw: Vec<_> = Vec::from(idxs);
    let mut bw: Vec<Option<usize>> = (0..ct).map(|_| None).collect();
    for (i, j) in fw.iter().enumerate() {
        let j = *j;
        assert!(bw[j].is_none());
        bw[j] = Some(i);
    }
    let mut bw: Vec<_> = bw.into_iter().map(Option::unwrap).collect();

    // Now fix a slot at a time, maintaining fw/bw
    for i in 0..ct {
        let b = bw[i];
        let f = fw[i];
        if i == f {
            continue;
        }

        ts.swap(i, f);

        // previously: (... -> b -> i -> f -> ...)
        // now: (... -> b -> f -> ...) and (i)
        fw[b] = f;
        bw[f] = b;

        // Not read any more, but maintained for assertions at end.
        fw[i] = i;
        bw[i] = i;
    }

    for i in 0..ct {
        assert_eq!(i, fw[i]);
        assert_eq!(i, bw[i]);
    }
}
