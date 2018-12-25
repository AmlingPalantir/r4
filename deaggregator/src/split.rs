use DeaggregatorBe;
use record::Record;
use registry::ThreeStringArgs;
use std::sync::Arc;

pub struct Impl();

impl DeaggregatorBe for Impl {
    type Args = ThreeStringArgs;

    fn names() -> Vec<&'static str> {
        return vec!["split"];
    }

    fn deaggregate(a: &(Arc<str>, Arc<str>, Arc<str>), r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        let v = r.get_path(&a.0).expect_string();
        return v.split(&*a.1).map(|v| vec![(a.2.clone(), Record::from_str(v))]).collect();
    }
}
