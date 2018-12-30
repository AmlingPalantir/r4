use record::Record;
use record::RecordTrait;
use registry::args::OneStringArgs;
use std::cmp::Ord;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

#[derive(Clone)]
pub struct MaxState<K>(Option<(K, Record)>);

impl<K> Default for MaxState<K> {
    fn default() -> Self {
        return MaxState(None);
    }
}

impl<K: Ord> MaxState<K> {
    pub fn add(&mut self, k: K, v: Record) {
        if let Some((k1, _v1)) = &self.0 {
            if k1 >= &k {
                return;
            }
        }
        self.0 = Some((k, v));
    }

    pub fn finish(self) -> Record {
        return self.0.unwrap().1;
    }
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneStringArgs;
    type State = MaxState<Arc<str>>;

    fn names() -> Vec<&'static str> {
        return vec!["lmax"];
    }

    fn add(state: &mut MaxState<Arc<str>>, a: &Arc<str>, r: Record) {
        let v = r.get_path(a);
        state.add(v.expect_string(), v);
    }

    fn finish(state: MaxState<Arc<str>>, _a: &Arc<str>) -> Record {
        return state.finish();
    }
}
