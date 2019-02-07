use record::Record;
use std::sync::Arc;
use super::DeaggregatorBe;
use super::DeaggregatorRegistrant;

#[derive(RegistryArgs)]
pub(crate) struct Args {
    in_key: Arc<str>,
    out_key: Arc<str>,
}

pub(crate) type Impl = DeaggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl DeaggregatorBe for ImplBe {
    type Args = Args;

    fn names() -> Vec<&'static str> {
        return vec!["unarray", "unarr"];
    }

    fn help_msg() -> &'static str {
        return "output one record per array element of a value";
    }

    fn deaggregate(a: &Args, r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        return r.get_path(&a.in_key).expect_array().iter().map(|v| vec![(a.out_key.clone(), v.clone())]).collect();
    }
}
