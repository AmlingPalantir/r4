use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use super::SortBeFromSimple;
use super::SortRegistrant;
use super::SortSimpleBe;

pub type Impl = SortRegistrant<ImplBe>;

pub(crate) type ImplBe = SortBeFromSimple<ImplSimpleBe>;

pub struct ImplSimpleBe;

impl SortSimpleBe for ImplSimpleBe {
    type T = Arc<str>;

    fn names() -> Vec<&'static str> {
        return vec!["lexical", "lex", "l"];
    }

    fn get(r: Record) -> Arc<str> {
        return r.coerce_string();
    }
}
