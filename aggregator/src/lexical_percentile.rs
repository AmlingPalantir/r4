use record::Record;
use record::RecordTrait;
use registry::RegistryArgs;
use std::cmp::Ord;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

#[derive(Clone)]
pub struct PercentileState<K>(Vec<(K, Record)>);

impl<K: Ord> Default for PercentileState<K> {
    fn default() -> Self {
        return PercentileState(Vec::new());
    }
}

impl<K: Ord> PercentileState<K> {
    pub fn add(&mut self, k: K, v: Record) {
        self.0.push((k, v));
    }

    pub fn finish(mut self, prop: f64) -> Record {
        self.0.sort_by(|(k1, _v1), (k2, _v2)| k1.cmp(k2));
        let mut idx = ((self.0.len() as f64) * prop) as usize;
        if idx >= self.0.len() {
            idx = self.0.len() - 1;
        }
        return self.0[idx].1.clone();
    }
}

pub enum PercentileArgs {
}

impl RegistryArgs for PercentileArgs {
    type Val = (f64, Arc<str>);

    fn argct() -> usize {
        return 2;
    }

    fn parse(args: &[&str]) -> (f64, Arc<str>) {
        assert_eq!(2, args.len());
        let prop = args[0].parse::<f64>().unwrap() / 100.0;
        assert!(0.0 <= prop && prop <= 1.0);
        return (prop, Arc::from(&*args[1]));
    }
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = PercentileArgs;
    type State = PercentileState<Arc<str>>;

    fn names() -> Vec<&'static str> {
        return vec!["lperc"];
    }

    fn add(state: &mut PercentileState<Arc<str>>, a: &(f64, Arc<str>), r: Record) {
        let v = r.get_path(&a.1);
        state.add(v.expect_string(), v);
    }

    fn finish(state: Box<PercentileState<Arc<str>>>, a: &(f64, Arc<str>)) -> Record {
        return state.finish(a.0);
    }
}
