use ClumperOptions;
use OperationBe;
use SubOperationOption;
use opts::OptParserView;
use stream::Stream;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["multiplex"];
}

#[derive(Default)]
pub struct Impl {
}

declare_opts! {
    cl: ClumperOptions,
    op: SubOperationOption,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
        opt.sub(|p| &mut p.cl).match_single(&["c", "clumper"], ClumperOptions::add_single);
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return &o.op.extra;
    }

    fn stream(o: &PostOptions) -> Stream {
        let sub_wr = o.op.wr.clone();
        return o.cl.stream(move |bucket| {
            let s = Stream::transform_records(move |mut r| {
                for (path, v) in &bucket {
                    r.set_path(&path, v.clone());
                }
                return r;
            });
            return Stream::compound(sub_wr.stream(), s);
        });
    }
}
