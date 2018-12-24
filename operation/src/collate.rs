use OperationBe;
use clumper::ClumperWrapper;
use opts::parser::OptParserView;
use opts::vals::UnvalidatedOption;
use stream::Stream;
use super::aggregate;

pub struct Impl();

declare_opts! {
    cws: UnvalidatedOption<Vec<Box<ClumperWrapper>>>,
    ag: <aggregate::Impl as OperationBe>::PreOptions,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["collate"];
    }

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        super::clumper_options(opt.sub(|p| &mut p.cws));
        aggregate::Impl::options(opt.sub(|p| &mut p.ag));
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return aggregate::Impl::get_extra(&o.ag);
    }

    fn stream(o: &PostOptions) -> Stream {
        let ag_opt = o.ag.clone();
        return clumper::stream(&o.cws, move |bucket| {
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
