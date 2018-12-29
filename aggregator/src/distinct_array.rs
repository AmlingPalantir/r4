use record::Record;
use record::RecordTrait;
use registry::OneStringArgs;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use super::AggregatorBe;

pub struct Impl();

#[derive(Clone)]
pub struct DistinctSet<T> {
    v: Vec<T>,
    m: HashMap<T, ()>,
}

impl<T: Eq + Hash> Default for DistinctSet<T> {
    fn default() -> Self {
        return DistinctSet {
            v: Vec::new(),
            m: HashMap::new(),
        };
    }
}

impl<T: Clone + Eq + Hash> DistinctSet<T> {
    pub fn add(&mut self, t: T) {
        if self.m.insert(t.clone(), ()).is_none() {
            self.v.push(t);
        }
    }
}

impl<T> IntoIterator for DistinctSet<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> std::vec::IntoIter<T> {
        return self.v.into_iter();
    }
}

impl AggregatorBe for Impl {
    type Args = OneStringArgs;
    type State = DistinctSet<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["darr", "darray"];
    }

    fn add(state: &mut DistinctSet<Record>, a: &Arc<str>, r: Record) {
        state.add(r.get_path(&a));
    }

    fn finish(state: Box<DistinctSet<Record>>, _a: &Arc<str>) -> Record {
        return Record::from_vec(state.into_iter().collect());
    }
}
