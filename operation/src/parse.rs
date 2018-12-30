use opts::parser::OptParserView;
use opts::vals::EmptyOption;
use std::sync::Arc;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl OperationBe2 for ImplBe2 {
    type Options = EmptyOption;

    fn names() -> Vec<&'static str> {
        return vec!["parse"];
    }

    fn options<'a>(_opt: &mut OptParserView<'a, EmptyOption>) {
    }

    fn stream(_o: Arc<()>) -> Stream {
        return stream::parse();
    }
}
