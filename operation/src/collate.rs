use opts::parser::OptParserView;
use opts::vals::IntoArcOption;
use std::sync::Arc;
use stream::Stream;
use super::ClumperOptions;
use super::OperationBe;
use super::OperationRegistrant;
use super::aggregate;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    cl: ClumperOptions,
    ag: IntoArcOption<<aggregate::ImplBe as OperationBe>::Options>,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl OperationBe for ImplBe {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["collate"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        ClumperOptions::options(&mut opt.sub(|p| &mut p.cl));
        aggregate::ImplBe::options(&mut opt.sub(|p| &mut p.ag.0));
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return aggregate::ImplBe::get_extra(o.ag.clone());
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
            return stream::compound(aggregate::ImplBe::stream(o2.ag.clone()), s);
        });
    }
}
