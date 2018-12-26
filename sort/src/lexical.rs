use SortSimpleBe;
use SortSimpleBeImpl;
use record::Record;
use std::sync::Arc;

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
