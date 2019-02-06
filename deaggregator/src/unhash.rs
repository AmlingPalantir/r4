use record::Record;
use registry::args::ThreeStringArgs;
use std::sync::Arc;
use super::DeaggregatorBe;
use super::DeaggregatorRegistrant;

pub(crate) type Impl = DeaggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl DeaggregatorBe for ImplBe {
    type Args = ThreeStringArgs;

    fn names() -> Vec<&'static str> {
        return vec!["unhash"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("in_key,key_key,value_key");
    }

    fn help_msg() -> &'static str {
        return "output one record per hash element of a value";
    }

    fn deaggregate(a: &(Arc<str>, Arc<str>, Arc<str>), r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        return r.get_path(&a.0).expect_hash().iter().map(|(k, v)| {
            return vec![
                (a.1.clone(), Record::from(k.clone())),
                (a.2.clone(), v.clone()),
            ];
        }).collect();
    }
}
