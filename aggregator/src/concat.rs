use AggregatorBe;
use record::Record;
use registry::TwoStringArgs;
use std::sync::Arc;

pub struct Impl();

impl AggregatorBe for Impl {
    type Args = TwoStringArgs;
    type State = Vec<String>;

    fn names() -> Vec<&'static str> {
        return vec!["concat", "concatenate"];
    }

    fn add(state: &mut Vec<String>, a: &(Arc<str>, Arc<str>), r: Record) {
        state.push(r.get_path(&a.1).expect_string().to_string());
    }

    fn finish(state: Box<Vec<String>>, a: &(Arc<str>, Arc<str>)) -> Record {
        return Record::from_primitive_string(state.join(&a.0));
    }
}
