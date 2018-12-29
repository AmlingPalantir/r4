use ClumperOptions;
use OperationBe;
use opts::parser::OptParserView;
use opts::vals::IntoArcOption;
use std::sync::Arc;
use stream::Stream;
use super::aggregate;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    cl: ClumperOptions,
    ag: IntoArcOption<<aggregate::Impl as OperationBe>::Options>,
}

impl OperationBe for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["collate"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        ClumperOptions::options(&mut opt.sub(|p| &mut p.cl));
        aggregate::Impl::options(&mut opt.sub(|p| &mut p.ag.0));
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return aggregate::Impl::get_extra(o.ag.clone());
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
            return stream::compound(aggregate::Impl::stream(o2.ag.clone()), s);
        });
    }
}
