use record::Record;
use registry::TwoStringArgs;
use std::sync::Arc;
use super::DeaggregatorBe;
use super::DeaggregatorRegistrant;

pub(crate) type Impl = DeaggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl DeaggregatorBe for ImplBe {
    type Args = TwoStringArgs;

    fn names() -> Vec<&'static str> {
        return vec!["unarray", "unarr"];
    }

    fn deaggregate(a: &(Arc<str>, Arc<str>), r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        return r.get_path(&a.0).expect_array().iter().map(|v| vec![(a.1.clone(), v.clone())]).collect();
    }
}
