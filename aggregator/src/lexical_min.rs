use AggregatorBe;
use record::Record;
use record::RecordTrait;
use registry::OneStringArgs;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::sync::Arc;
use super::lexical_max::MaxState;

pub struct Impl();

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

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = MaxState<ReverseOrd<Arc<str>>>;

    fn names() -> Vec<&'static str> {
        return vec!["lmin"];
    }

    fn add(state: &mut MaxState<ReverseOrd<Arc<str>>>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(ReverseOrd(v.expect_string()), v);
    }

    fn finish(state: Box<MaxState<ReverseOrd<Arc<str>>>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
