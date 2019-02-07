use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use super::DeaggregatorBe;
use super::DeaggregatorRegistrant;

#[derive(RegistryArgs)]
pub(crate) struct Args {
    in_key: Arc<str>,
    delimiter: Arc<str>,
    out_key: Arc<str>,
}

pub(crate) type Impl = DeaggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl DeaggregatorBe for ImplBe {
    type Args = Args;

    fn names() -> Vec<&'static str> {
        return vec!["split"];
    }

    fn help_meta() -> Option<&'static str> {
        return Some("in_key,delimiter,out_key");
    }

    fn help_msg() -> &'static str {
        return "split one value and output one record per split piece";
    }

    fn deaggregate(a: &Args, r: Record) -> Vec<Vec<(Arc<str>, Record)>> {
        let v = r.get_path(&a.in_key).expect_string();
        return v.split(&*a.delimiter).map(|v| vec![(a.out_key.clone(), Record::from(v))]).collect();
    }
}
