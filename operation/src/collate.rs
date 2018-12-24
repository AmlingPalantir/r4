use ClumperOptions;
use OperationBe;
use opts::parser::OptParserView;
use stream::Stream;
use super::aggregate;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["collate"];
}

#[derive(Default)]
pub struct Impl {
}

declare_opts! {
    cl: ClumperOptions,
    ag: <aggregate::Impl as OperationBe>::PreOptions,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.cl).match_single(&["c", "clumper"], ClumperOptions::add_single);
        aggregate::Impl::options(opt.sub(|p| &mut p.ag));
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return aggregate::Impl::get_extra(&o.ag);
    }

    fn stream(o: &PostOptions) -> Stream {
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
