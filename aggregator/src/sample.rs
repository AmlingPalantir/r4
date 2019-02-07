use rand::Rng;
use record::Record;
use record::RecordTrait;
use registry_args::RegistryArgs;
use std::cmp::Ord;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;
use validates::ValidationResult;

pub enum SampleArgs {
}

impl RegistryArgs for SampleArgs {
    type Val = (usize, Arc<str>);

    fn argct() -> usize {
        return 2;
    }

    fn parse(args: &[&str]) -> ValidationResult<(usize, Arc<str>)> {
        assert_eq!(2, args.len());
        return Result::Ok((args[0].parse()?, Arc::from(&*args[1])));
    }
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = SampleArgs;
    type State = (usize, Vec<(usize, Record)>);

    fn names() -> Vec<&'static str> {
        return vec!["sample"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("count,key");
    }

    fn help_msg() -> &'static str {
        return "sample a specified number of values";
    }

    fn add(state: &mut (usize, Vec<(usize, Record)>), a: &(usize, Arc<str>), r: Record) {
        let idx = state.0;
        state.0 += 1;

        let v = r.get_path(&a.1);
        if state.1.len() < a.0 {
            state.1.push((idx, v));
            return;
        }
        let pos = rand::thread_rng().gen_range(0, idx + 1);
        if pos < a.0 {
            state.1[pos] = (idx, v);
        }
    }

    fn finish(mut state: (usize, Vec<(usize, Record)>), _a: &(usize, Arc<str>)) -> Record {
        state.1.sort_by(|(idx1, _v1), (idx2, _v2)| idx1.cmp(idx2));
        return Record::from_vec(state.1.into_iter().map(|(_idx, r)| r).collect());
    }
}
