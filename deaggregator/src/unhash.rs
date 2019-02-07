use record::Record;
use std::sync::Arc;
use super::DeaggregatorBe;
use super::DeaggregatorRegistrant;

#[derive(RegistryArgs)]
pub(crate) struct Args {
    in_key: Arc<str>,
    key_key: Arc<str>,
    value_key: Arc<str>,
}

pub(crate) type Impl = DeaggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl DeaggregatorBe for ImplBe {
    type Args = Args;

    fn names() -> Vec<&'static str> {
        return vec!["unhash"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("in_key,key_key,value_key");
    }

    fn help_msg() -> &'static str {
        return "output one record per hash element of a value";
    }

    fn deaggregate(a: &Args, r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        return r.get_path(&a.in_key).expect_hash().iter().map(|(k, v)| {
            return vec![
                (a.key_key.clone(), Record::from(k.clone())),
                (a.value_key.clone(), v.clone()),
            ];
        }).collect();
    }
}
