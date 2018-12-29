use ClumperOptions;
use OperationBe;
use opts::parser::OptParserView;
use stream::Stream;
use super::aggregate;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    cl: ClumperOptions,
    ag: <aggregate::Impl as OperationBe>::Options,
}

impl OperationBe for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["collate"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        ClumperOptions::options(&mut opt.sub(|p| &mut p.cl));
        aggregate::Impl::options(&mut opt.sub(|p| &mut p.ag));
    }

    fn get_extra(o: &OptionsValidated) -> Vec<String> {
        return aggregate::Impl::get_extra(&o.ag);
    }

    fn stream(o: &OptionsValidated) -> Stream {
        let ag_opt = o.ag.clone();
        return o.cl.stream(move |bucket| {
            let s = stream::transform_records(move |mut r| {
                for (path, v) in &bucket {
                    r.set_path(&path, v.clone());
                }
                return r;
            });
            return stream::compound(aggregate::Impl::stream(&ag_opt), s);
        });
    }
}
