use rand::seq::SliceRandom;
use record::Record;
use registry::ZeroArgs;
use super::SortBe;

pub struct Impl();

impl SortBe for Impl {
    type Args = ZeroArgs;

    fn names() -> Vec<&'static str> {
        return vec!["shuffle"];
    }

    fn sort(_a: &(), rs: &mut [Record]) {
        rs.shuffle(&mut rand::thread_rng());
    }

    fn sort_aux<'a>(_a: &(), ct: usize, _f: Box<Fn(usize) -> &'a Record + 'a>) -> Vec<usize> {
        let mut idxs: Vec<_> = (0..ct).collect();
        idxs.shuffle(&mut rand::thread_rng());
        return idxs;
    }
}
