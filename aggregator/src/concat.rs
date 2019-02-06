use record::Record;
use record::RecordTrait;
use registry::args::TwoStringArgs;
use std::sync::Arc;
use super::AggregatorBe;
use super::AggregatorRegistrant;

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = TwoStringArgs;
    type State = Vec<String>;

    fn names() -> Vec<&'static str> {
        return vec!["concatenate", "concat"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("delim,key");
    }

    fn help_msg() -> &'static str {
        return "collect values into a string joined by a delimter";
    }

    fn add(state: &mut Vec<String>, a: &(Arc<str>, Arc<str>), r: Record) {
        state.push(r.get_path(&a.1).expect_string().to_string());
    }

    fn finish(state: Vec<String>, a: &(Arc<str>, Arc<str>)) -> Record {
        return Record::from(state.join(&a.0));
    }
}
