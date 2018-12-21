use Aggregator0;
use record::FromPrimitive;
use record::Record;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["ct", "count"];
}

#[derive(Clone)]
#[derive(Default)]
pub struct Impl {
    ct: u32,
}

impl Aggregator0 for Impl {
    fn add(&mut self, _r: Record) {
        self.ct += 1;
    }

    fn finish(self) -> Record {
        return Record::from_primitive(self.ct);
    }
}
