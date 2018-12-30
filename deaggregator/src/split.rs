use record::Record;
use record::RecordTrait;
use registry::ThreeStringArgs;
use std::sync::Arc;
use super::DeaggregatorBe;
use super::DeaggregatorRegistrant;

pub(crate) type Impl = DeaggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl DeaggregatorBe for ImplBe {
    type Args = ThreeStringArgs;

    fn names() -> Vec<&'static str> {
        return vec!["split"];
    }

    fn deaggregate(a: &(Arc<str>, Arc<str>, Arc<str>), r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        let v = r.get_path(&a.0).expect_string();
        return v.split(&*a.1).map(|v| vec![(a.2.clone(), Record::from(v))]).collect();
    }
}
