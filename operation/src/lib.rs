extern crate aggregator;
extern crate bgop;
extern crate clumper;
extern crate deaggregator;
#[macro_use]
extern crate lazy_static;
extern crate opts;
extern crate record;
extern crate regex;
#[macro_use]
extern crate registry;
extern crate stream;
extern crate stream_process;
extern crate validates;
#[macro_use]
extern crate validates_derive;

registry! {
    OperationFe,
    Box<Fn(&mut Vec<String>) -> StreamWrapper>,
    aggregate,
    bg,
    chain,
    collate,
    decollate,
    from_lines,
    from_multi_regex,
    from_regex,
    from_split,
    join,
    multiplex,
    test,
    to_table,
    with_files,
    with_lines,
}

use clumper::ClumperFe;
use clumper::ClumperWrapper;
use opts::parser::OptParser;
use opts::parser::OptParserView;
use opts::vals::OptionalStringOption;
use record::Record;
use std::rc::Rc;
use std::sync::Arc;
use stream::Stream;
use validates::Validates;

pub trait OperationFe {
    fn names() -> Vec<&'static str>;
    fn argct() -> usize;
    fn init(args: &[&str]) -> Box<Fn(&mut Vec<String>) -> StreamWrapper>;
}

pub trait OperationFe2 {
    fn validate(&self, &mut Vec<String>) -> StreamWrapper;
}

pub struct StreamWrapper(Box<Fn() -> Stream + Send + Sync>);

impl StreamWrapper {
    pub fn new<F: Fn() -> Stream + Send + Sync + 'static>(f: F) -> Self {
        return StreamWrapper(Box::new(f));
    }

    pub fn stream(&self) -> Stream {
        return self.0();
    }
}



pub trait OperationBe {
    type PreOptions: Validates<Target = Self::PostOptions> + Default + 'static;
    type PostOptions: Send + Sync + 'static;

    fn names() -> Vec<&'static str>;
    fn options<'a>(&mut OptParserView<'a, Self::PreOptions>);
    fn get_extra(&Self::PostOptions) -> &Vec<String>;
    fn stream(&Self::PostOptions) -> Stream;
}

impl<B: OperationBe> OperationFe for B {
    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn argct() -> usize {
        return 0;
    }

    fn init(_args: &[&str]) -> Box<Fn(&mut Vec<String>) -> StreamWrapper> {
        return Box::new(|args| {
            let mut opt = OptParser::<B::PreOptions>::new();
            B::options(&mut opt.view());
            let o = opt.parse(args).validate();
            *args = B::get_extra(&o).clone();

            return StreamWrapper::new(move || B::stream(&o));
        });
    }
}



pub trait OperationBe2 {
    type PreOptions: Validates<Target = Self::PostOptions> + Default + 'static;
    type PostOptions: Send + Sync + 'static;

    fn names() -> Vec<&'static str>;
    fn options<'a>(&mut OptParserView<'a, Self::PreOptions>);
    fn stream(&Self::PostOptions) -> Stream;
}

#[derive(Clone)]
#[derive(Default)]
pub struct AndArgsOptions<P> {
    p: P,
    args: Vec<String>,
}

impl<P: Validates> Validates for AndArgsOptions<P> {
    type Target = AndArgsOptions<<P as Validates>::Target>;

    fn validate(self) -> AndArgsOptions<<P as Validates>::Target> {
        return AndArgsOptions {
            p: self.p.validate(),
            args: self.args,
        };
    }
}

impl<B: OperationBe2> OperationBe for B {
    type PreOptions = AndArgsOptions<B::PreOptions>;
    type PostOptions = AndArgsOptions<B::PostOptions>;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn options<'a>(opt: &mut OptParserView<'a, AndArgsOptions<B::PreOptions>>) {
        B::options(&mut opt.sub(|p| &mut p.p));
        opt.sub(|p| &mut p.args).match_extra_soft(|p, a| {
            p.push(a.to_string());
            return true;
        });
    }

    fn get_extra(p: &AndArgsOptions<B::PostOptions>) -> &Vec<String> {
        return &p.args;
    }

    fn stream(p: &AndArgsOptions<B::PostOptions>) -> Stream {
        return B::stream(&p.p);
    }
}

#[derive(Default)]
struct SubOperationOption(Vec<String>);

impl SubOperationOption {
    fn push(&mut self, a: &[String]) {
        self.0.extend_from_slice(a);
    }

    fn of(a: Vec<String>) -> SubOperationOption {
        return SubOperationOption(a);
    }
}

impl Validates for SubOperationOption {
    type Target = SubOperationOptions;

    fn validate(mut self) -> SubOperationOptions {
        if self.0.len() >= 2 && self.0[0] == "r4" {
            self.0.remove(0);
            let name = self.0.remove(0);
            let op = REGISTRY.find(&name, &[]);
            let wr = op(&mut self.0);
            return SubOperationOptions {
                extra: self.0,
                wr: Arc::new(wr),
            };
        }

        return SubOperationOptions {
            extra: vec![],
            wr: Arc::new(StreamWrapper::new(move || {
                return stream_process::new(self.0.clone());
            })),
        };
    }
}

#[derive(Clone)]
struct SubOperationOptions {
    extra: Vec<String>,
    wr: Arc<StreamWrapper>,
}

#[derive(Clone)]
#[derive(Default)]
struct ClumperOptions {
    cws: Vec<Box<ClumperWrapper>>,
}

impl Validates for ClumperOptions {
    type Target = ClumperOptions;

    fn validate(self) -> ClumperOptions {
        return self;
    }
}

impl ClumperOptions {
    fn options<'a>(opt: &mut OptParserView<'a, ClumperOptions>) {
        clumper::REGISTRY.single_options(&mut opt.sub(|p| &mut p.cws), &["c", "clumper"]);
        clumper::REGISTRY.multiple_options(&mut opt.sub(|p| &mut p.cws), &["c", "clumper"]);
        opt.match_single(&["k", "key"], |p, a| {
            for a in a.split(',') {
                p.cws.push(clumper::key::Impl::init(&[a]));
            }
        });
    }

    fn stream<F: Fn(Vec<(Arc<str>, Record)>) -> Stream + 'static>(&self, f: F) -> Stream {
        let mut bsw: Rc<Fn(Vec<(Arc<str>, Record)>) -> Stream> = Rc::new(f);

        bsw = self.cws.iter().rev().fold(bsw, |bsw, cw| {
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

#[derive(Default)]
#[derive(Validates)]
struct TwoRecordUnionOption {
    left_prefix: OptionalStringOption,
    right_prefix: OptionalStringOption,
}

impl TwoRecordUnionOption {
    fn options<'a>(opt: &mut OptParserView<'a, TwoRecordUnionOption>) {
        opt.sub(|p| &mut p.left_prefix).match_single(&["lp", "left-prefix"], OptionalStringOption::set);
        opt.sub(|p| &mut p.right_prefix).match_single(&["rp", "right-prefix"], OptionalStringOption::set);
    }
}

impl TwoRecordUnionOptionValidated {
    fn union_maybe(&self, r1: Option<Record>, r2: Option<Record>) -> Record {
        fn _union_aux(r: &mut Record, prefix: &Option<Arc<str>>, r1: Record) {
            match prefix {
                Some(prefix) => {
                    r.set_path(&prefix, r1);
                }
                None => {
                    for (k, v) in r1.expect_hash().into_iter() {
                        r.set_path(&k, v.clone());
                    }
                }
            }
        }

        let mut r = Record::empty_hash();
        if let Some(r1) = r1 {
            _union_aux(&mut r, &self.left_prefix, r1);
        }
        if let Some(r2) = r2 {
            _union_aux(&mut r, &self.right_prefix, r2);
        }
        return r;
    }

    fn union(&self, r1: Record, r2: Record) -> Record {
        return self.union_maybe(Some(r1), Some(r2));
    }
}
