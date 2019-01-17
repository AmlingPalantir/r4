use clumper::BoxedClumper;
use opts::parser::OptParserView;
use opts::vals::UnvalidatedOption;
use record::Record;
use registry::Registrant;
use std::rc::Rc;
use std::sync::Arc;
use stream::Stream;

#[derive(Default)]
#[derive(Validates)]
pub struct ClumperOptions(UnvalidatedOption<Vec<BoxedClumper>>);

impl ClumperOptions {
    pub fn options<'a>(opt: &mut OptParserView<'a, ClumperOptions>) {
        clumper::REGISTRY.single_options(&mut opt.sub(|p| &mut (p.0).0), &["c", "clumper"]);
        clumper::REGISTRY.multiple_options(&mut opt.sub(|p| &mut (p.0).0), &["c", "clumper"]);
        opt.match_single(&["k", "key"], |p, a| {
            for a in a.split(',') {
                (p.0).0.push(clumper::key::Impl::init(&[a]));
            }
            return Result::Ok(());
        });
    }
}

impl ClumperOptionsValidated {
    pub fn stream<F: Fn(Vec<(Arc<str>, Record)>) -> Stream + 'static>(&self, f: F) -> Stream {
        let mut bsw: Rc<Fn(Vec<(Arc<str>, Record)>) -> Stream> = Rc::new(f);

        bsw = self.0.iter().rev().fold(bsw, |bsw, cw| {
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
