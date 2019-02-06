use record::Record;
use registry::args::TwoStringArgs;
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

    fn help_meta() -> Option<&'static str> {
        return Some("in_key,out_key");
    }

    fn help_msg() -> &'static str {
        return "output one record per array element of a value";
    }

    fn deaggregate(a: &(Arc<str>, Arc<str>), r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        return r.get_path(&a.0).expect_array().iter().map(|v| vec![(a.1.clone(), v.clone())]).collect();
    }
}
