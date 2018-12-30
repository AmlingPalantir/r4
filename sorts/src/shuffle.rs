use rand::seq::SliceRandom;
use record::Record;
use registry::args::ZeroArgs;
use super::SortBe;
use super::SortRegistrant;

pub(crate) type Impl = SortRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl SortBe for ImplBe {
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
