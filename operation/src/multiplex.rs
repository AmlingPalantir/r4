use opts::parser::OptParserView;
use std::sync::Arc;
use stream::Stream;
use super::ClumperOptions;
use super::OperationBe;
use super::OperationRegistrant;
use super::SubOperationOption;
use validates::Validates;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    cl: ClumperOptions,
    op: SubOperationOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl OperationBe for ImplBe {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["multiplex"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
        ClumperOptions::options(&mut opt.sub(|p| &mut p.cl));
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return o.op.extra.clone();
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        let o2 = o.clone();
        return o.cl.stream(move |bucket| {
            let s = stream::transform_records(move |mut r| {
                for (path, v) in &bucket {
                    r.set_path(&path, v.clone());
                }
                return r;
            });
            return stream::compound(o2.op.wr.stream(), s);
        });
    }
}
