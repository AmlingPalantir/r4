use Aggregator0;
use record::Record;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["ct", "count"];
}

#[derive(Clone)]
#[derive(Default)]
pub struct Impl(Vec<Record>);

impl Aggregator0 for Impl {
    fn add(&mut self, r: Record) {
        self.0.push(r);
    }

    fn finish(self) -> Record {
        return Record::from_vec(self.0);
    }
}
