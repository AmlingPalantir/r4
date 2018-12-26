use OperationBe2;
use opts::parser::OptParserView;
use opts::vals::RequiredStringOption;
use record::Record;
use std::sync::Arc;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    msg: RequiredStringOption,
}

impl OperationBe2 for Impl {
    type PreOptions = Options;
    type PostOptions = OptionsValidated;

    fn names() -> Vec<&'static str> {
        return vec!["test"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.msg).match_single(&["m", "msg"], RequiredStringOption::set);
    }

    fn stream(o: &OptionsValidated) -> Stream {
        let msg: Arc<str> = Arc::from(&*o.msg);
        let mut n = 0;

        let s = stream::transform_records(move |mut r| {
            n += 1;
            r.set_path("n", Record::from_i64(n));
            r.set_path("msg", Record::from_str(msg.clone()));

            return r;
        });
        return stream::compound(stream::parse(), s);
    }
}
