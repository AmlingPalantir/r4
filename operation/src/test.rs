use OperationBe2;
use opts::parser::OptParserView;
use opts::vals::RequiredStringOption;
use record::FromPrimitive;
use record::Record;
use std::sync::Arc;
use stream::Stream;

pub struct Impl();

declare_opts! {
    msg: RequiredStringOption,
}

impl OperationBe2 for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["test"];
    }

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.msg).match_single(&["m", "msg"], RequiredStringOption::set);
    }

    fn stream(o: &PostOptions) -> Stream {
        let msg: Arc<str> = Arc::from(&*o.msg);
        let mut n = 0;

        let s = stream::transform_records(move |mut r| {
            n += 1;
            r.set_path("n", Record::from_primitive(n));
            r.set_path("msg", Record::from_primitive_string(msg.clone()));

            return r;
        });
        return stream::compound(stream::parse(), s);
    }
}
