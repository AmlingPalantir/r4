use record::Record;
use record::RecordTrait;
use registry_args::RegistryArg;
use std::cmp::Ord;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use validates::ValidationError;
use validates::ValidationResult;

#[derive(Clone)]
pub(crate) struct PercentileArg(f64);

impl Copy for PercentileArg {
}

impl RegistryArg for PercentileArg {
    fn parse(arg: &str) -> ValidationResult<PercentileArg> {
        let perc = arg.parse::<f64>()?;
        let prop = perc / 100.0;
        if !(0.0 <= prop && prop <= 1.0) {
            return ValidationError::message(format!("Percentile must be between 0 and 100: {}", prop));
        }
        return Result::Ok(PercentileArg(prop));
    }
}

#[derive(RegistryArgs)]
pub(crate) struct PercentileArgs {
    pub(crate) percentile: PercentileArg,
    pub(crate) key: Arc<str>,
}

#[derive(Clone)]
pub(crate) struct PercentileState<K>(Vec<(K, Record)>);

impl<K: Ord> Default for PercentileState<K> {
    fn default() -> Self {
        return PercentileState(Vec::new());
    }
}

impl<K: Ord> PercentileState<K> {
    pub fn add(&mut self, k: K, v: Record) {
        self.0.push((k, v));
    }

    pub fn finish(mut self, percentile: PercentileArg) -> Record {
        self.0.sort_by(|(k1, _v1), (k2, _v2)| k1.cmp(k2));
        let mut idx = ((self.0.len() as f64) * percentile.0) as usize;
        if idx >= self.0.len() {
            idx = self.0.len() - 1;
        }
        return self.0[idx].1.clone();
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

    fn help_msg() -> &'static str {
        return "compute a percentile of values sorted lexically";
    }

    fn add(state: &mut PercentileState<Arc<str>>, a: &PercentileArgs, r: Record) {
        let v = r.get_path(&a.key);
        state.add(v.expect_string(), v);
    }

    fn finish(state: PercentileState<Arc<str>>, a: &PercentileArgs) -> Record {
        return state.finish(a.percentile);
    }
}
