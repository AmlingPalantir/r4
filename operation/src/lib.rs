extern crate aggregator;
extern crate bgop;
extern crate clumper;
extern crate deaggregator;
extern crate executor;
#[macro_use]
extern crate lazy_static;
extern crate opts;
extern crate record;
extern crate regex;
#[macro_use]
extern crate registry;
extern crate sorts;
extern crate stream;
extern crate validates;
#[macro_use]
extern crate validates_derive;

mod tru;
pub(crate) use tru::TwoRecordUnionOption;

mod clumper_options;
pub(crate) use clumper_options::ClumperOptions;

mod subop_options;
pub(crate) use subop_options::SubOperationOption;

registry! {
    OperationFe,
    Box<Fn(&mut Vec<String>) -> StreamWrapper>,
    aggregate,
    bg,
    chain,
    collate,
    decollate,
    eval,
    from_lines,
    from_multi_regex,
    from_regex,
    from_split,
    grep,
    join,
    multiplex,
    shell,
    sort,
    test,
    to_ptable,
    to_table,
    with_files,
    with_lines,
    xform,
}

use opts::parser::OptParser;
use opts::parser::OptParserView;
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
    type Options: Validates + Default + 'static;

    fn names() -> Vec<&'static str>;
    fn options<'a>(&mut OptParserView<'a, Self::Options>);
    fn get_extra(&<Self::Options as Validates>::Target) -> Vec<String>;
    fn stream(&<Self::Options as Validates>::Target) -> Stream;
}

impl<B: OperationBe> OperationFe for B where <B::Options as Validates>::Target: Send + Sync {
    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn argct() -> usize {
        return 0;
    }

    fn init(_args: &[&str]) -> Box<Fn(&mut Vec<String>) -> StreamWrapper> {
        return Box::new(|args| {
            let mut opt = OptParser::<B::Options>::new();
            B::options(&mut opt.view());
            let o = opt.parse(args).validate();
            *args = B::get_extra(&o);

            return StreamWrapper::new(move || B::stream(&o));
        });
    }
}



pub trait OperationBe2 {
    type Options: Validates + Default + 'static;

    fn names() -> Vec<&'static str>;
    fn options<'a>(&mut OptParserView<'a, Self::Options>);
    fn stream(&<Self::Options as Validates>::Target) -> Stream;
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
    type Options = AndArgsOptions<B::Options>;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn options<'a>(opt: &mut OptParserView<'a, AndArgsOptions<B::Options>>) {
        B::options(&mut opt.sub(|p| &mut p.p));
        opt.sub(|p| &mut p.args).match_extra_soft(|p, a| {
            p.push(a.to_string());
            return true;
        });
    }

    fn get_extra(p: &AndArgsOptions<<B::Options as Validates>::Target>) -> Vec<String> {
        return p.args.clone();
    }

    fn stream(p: &AndArgsOptions<<B::Options as Validates>::Target>) -> Stream {
        return B::stream(&p.p);
    }
}
