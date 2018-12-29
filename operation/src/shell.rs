use OperationBe;
use opts::parser::OptParserView;
use opts::vals::StringVecOption;
use std::sync::Arc;
use stream::Stream;

pub struct Impl();

impl OperationBe for Impl {
    type PreOptions = StringVecOption;
    type PostOptions = Arc<Vec<String>>;

    fn names() -> Vec<&'static str> {
        return vec!["shell"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, StringVecOption>) {
        opt.match_extra_hard(StringVecOption::push_all);
    }

    fn get_extra(_o: &Arc<Vec<String>>) -> Vec<String> {
        return vec![];
    }

    fn stream(o: &Arc<Vec<String>>) -> Stream {
        return stream_process::new(o as &Vec<String>);
    }
}
