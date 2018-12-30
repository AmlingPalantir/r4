use record::Record;
use record::RecordTrait;
use registry::args::TwoStringArgs;
use std::collections::BTreeMap;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

#[derive(Clone)]
#[derive(Default)]
pub struct State {
    s1: f64,
    sx: f64,
    sx2: f64,
    sy: f64,
    sy2: f64,
    sxy: f64,
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = TwoStringArgs;
    type State = State;

    fn names() -> Vec<&'static str> {
        return vec!["linreg"];
    }

    fn add(state: &mut State, a: &(Arc<str>, Arc<str>), r: Record) {
        let x = r.get_path(&a.0).coerce_f64();
        let y = r.get_path(&a.1).coerce_f64();
        state.s1 += 1.0;
        state.sx += x;
        state.sx2 += x * x;
        state.sy += y;
        state.sy2 += y * y;
        state.sxy += x * y;
    }

    fn finish(state: State, _a: &(Arc<str>, Arc<str>)) -> Record {
        let beta = (state.sxy * state.s1 - state.sx * state.sy) / (state.sx2 * state.s1 - state.sx * state.sx);
        let alpha = (state.sy - beta * state.sx) / state.s1;

        let sbeta_numerator = (state.sy2 + alpha * alpha * state.s1 + beta * beta * state.sx2 - 2.0 * alpha * state.sy + 2.0 * alpha * beta * state.sx - 2.0 * beta * state.sxy) / (state.s1 - 2.0);
        let sbeta_denominator = state.sx2 - state.sx * state.sx / state.s1;
        let sbeta = (sbeta_numerator / sbeta_denominator).sqrt();
        let salpha = sbeta * (state.sx2 / state.s1).sqrt();

        let mut hash = BTreeMap::new();

        hash.insert(Arc::from("alpha"), Record::from(alpha));
        hash.insert(Arc::from("beta"), Record::from(beta));
        hash.insert(Arc::from("alpha_se"), Record::from(salpha));
        hash.insert(Arc::from("beta_se"), Record::from(sbeta));

        return Record::from_hash(hash);
    }
}
