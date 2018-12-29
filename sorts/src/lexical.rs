use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use super::SortSimpleBe;
use super::SortSimpleBeImpl;

pub type Impl = SortSimpleBeImpl<SimpleImpl>;

pub struct SimpleImpl();

impl SortSimpleBe for SimpleImpl {
    type T = Arc<str>;

    fn names() -> Vec<&'static str> {
        return vec!["l", "lex", "lexical"];
    }

    fn get(r: Record) -> Arc<str> {
        return r.coerce_string();
    }
}
