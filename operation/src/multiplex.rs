use OperationBe;
use SubOperationOption;
use clumper::ClumperWrapper;
use opts::OptParserView;
use opts::UnvalidatedOption;
use opts::VarOption;
use record::Record;
use std::rc::Rc;
use std::sync::Arc;
use stream::Stream;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["multiplex"];
}

#[derive(Default)]
pub struct Impl {
}

declare_opts! {
    cws: UnvalidatedOption<Vec<Arc<Box<ClumperWrapper>>>>,
    op: SubOperationOption,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.op).match_extra_hard(VarOption::push_string_vec);
        opt.sub(|p| &mut p.cws).match_single(&["c", "clumper"], |cws, a| {
            let mut parts = a.split(',');
            let cl = clumper::find(parts.next().unwrap());
            let args: Vec<&str> = parts.collect();
            let cw = cl.wrapper(&args);
            cws.push(Arc::new(cw));
        });
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return &o.op.0;
    }

    fn stream(o: &PostOptions) -> Stream {
        let sub = o.op.1.clone();
        let mut bsw: Rc<Fn(Vec<(Arc<str>, Record)>) -> Stream> = Rc::new(move |bucket| {
            let s = Stream::transform_records(move |mut r| {
                for (path, v) in &bucket {
                    r.set_path(&path, v.clone());
                }
                return r;
            });
            return Stream::compound(sub.stream(), s);
        });

        bsw = o.cws.iter().rev().fold(bsw, |bsw, cw| {
            let cw = cw.clone();
            return Rc::new(move |bucket_outer| {
                let bucket_outer = bucket_outer.clone();
                let bsw = bsw.clone();
                return cw.stream(Box::new(move |bucket_inner| {
                    let mut bucket = bucket_outer.clone();
                    bucket.extend(bucket_inner);
                    return bsw(bucket);
                }));
            });
        });

        return bsw(vec![]);
    }
}
