use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

#[derive(RegistryArgs)]
pub(crate) struct Args {
    delimiter: Arc<str>,
    key: Arc<str>,
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = Args;
    type State = Vec<String>;

    fn names() -> Vec<&'static str> {
        return vec!["concatenate", "concat"];
    }

    fn help_msg() -> &'static str {
        return "collect values into a string joined by a delimter";
    }

    fn add(state: &mut Vec<String>, a: &Args, r: Record) {
        state.push(r.get_path(&a.key).expect_string().to_string());
    }

    fn finish(state: Vec<String>, a: &Args) -> Record {
        return Record::from(state.join(&a.delimiter));
    }
}
