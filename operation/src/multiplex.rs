use ClumperOptions;
use OperationBe;
use SubOperationOption;
use opts::parser::OptParserView;
use stream::Stream;

pub struct Impl();

declare_opts! {
    cl: ClumperOptions,
    op: SubOperationOption,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["multiplex"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
        ClumperOptions::options(&mut opt.sub(|p| &mut p.cl));
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return &o.op.extra;
    }

    fn stream(o: &PostOptions) -> Stream {
        let sub_wr = o.op.wr.clone();
        return o.cl.stream(move |bucket| {
            let s = stream::transform_records(move |mut r| {
                for (path, v) in &bucket {
                    r.set_path(&path, v.clone());
                }
                return r;
            });
            return stream::compound(sub_wr.stream(), s);
        });
    }
}
