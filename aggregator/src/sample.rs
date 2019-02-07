use rand::Rng;
use record::Record;
use record::RecordTrait;
use std::cmp::Ord;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

#[derive(RegistryArgs)]
pub(crate) struct Args {
    count: usize,
    key: Arc<str>,
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = Args;
    type State = (usize, Vec<(usize, Record)>);

    fn names() -> Vec<&'static str> {
        return vec!["sample"];
    }

    fn help_msg() -> &'static str {
        return "sample a specified number of values";
    }

    fn add(state: &mut (usize, Vec<(usize, Record)>), a: &Args, r: Record) {
        let idx = state.0;
        state.0 += 1;

        let v = r.get_path(&a.key);
        if state.1.len() < a.count {
            state.1.push((idx, v));
            return;
        }
        let pos = rand::thread_rng().gen_range(0, idx + 1);
        if pos < a.count {
            state.1[pos] = (idx, v);
        }
    }

    fn finish(mut state: (usize, Vec<(usize, Record)>), _a: &Args) -> Record {
        state.1.sort_by(|(idx1, _v1), (idx2, _v2)| idx1.cmp(idx2));
        return Record::from_vec(state.1.into_iter().map(|(_idx, r)| r).collect());
    }
}
