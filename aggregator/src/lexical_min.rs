use record::Record;
use record::RecordTrait;
use registry::args::OneKeyRegistryArgs;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use super::lexical_max::MaxState;

#[derive(Clone)]
#[derive(Eq)]
#[derive(PartialEq)]
pub struct ReverseOrd<T>(pub T);

impl<T: PartialOrd> PartialOrd for ReverseOrd<T> {
    fn partial_cmp(&self, other: &ReverseOrd<T>) -> Option<Ordering> {
        return self.0.partial_cmp(&other.0).map(Ordering::reverse);
    }
}

impl<T: Ord> Ord for ReverseOrd<T> {
    fn cmp(&self, other: &ReverseOrd<T>) -> Ordering {
        return self.0.cmp(&other.0).reverse();
    }
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneKeyRegistryArgs;
    type State = MaxState<ReverseOrd<Arc<str>>>;

    fn names() -> Vec<&'static str> {
        return vec!["lmin"];
    }

    fn help_msg() -> &'static str {
        return "track the lexically minimal value";
    }

    fn add(state: &mut MaxState<ReverseOrd<Arc<str>>>, a: &OneKeyRegistryArgs, r: Record) {
        let v = r.get_path(&a.key);
        state.add(ReverseOrd(v.expect_string()), v);
    }

    fn finish(state: MaxState<ReverseOrd<Arc<str>>>, _a: &OneKeyRegistryArgs) -> Record {
        return state.finish();
    }
}
