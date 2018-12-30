extern crate aggregator;
extern crate bgop;
extern crate clumper;
extern crate deaggregator;
extern crate executor;
#[macro_use]
extern crate lazy_static;
#[macro_use]
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
pub(crate) use self::tru::TwoRecordUnionOption;

mod clumper_options;
pub(crate) use self::clumper_options::ClumperOptions;

mod subop_options;
pub(crate) use self::subop_options::SubOperationOption;

use opts::parser::OptParser;
use opts::parser::OptParserView;
use opts::vals::IntoArcOption;
use opts::vals::StringVecOption;
use registry::Registrant;
use registry::args::ZeroArgs;
use std::sync::Arc;
use stream::Stream;
use validates::Validates;

pub type BoxedOperation = Box<OperationInbox>;

registry! {
    BoxedOperation,
    aggregate,
    bg,
    chain,
    collate,
    decollate,
    deparse,
    eval,
    from_lines,
    from_multi_regex,
    from_regex,
    from_split,
    grep,
    join,
    multiplex,
    parse,
    provenance,
    shell,
    sort,
    to_ptable,
    to_table,
    with_files,
    with_lines,
    xform,
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
    fn options<'a>(opt: &mut OptParserView<'a, Self::Options>);
    fn get_extra(o: Arc<<Self::Options as Validates>::Target>) -> Vec<String>;
    fn stream(o: Arc<<Self::Options as Validates>::Target>) -> Stream;
}

pub trait OperationInbox {
    fn parse(&self, args: &mut Vec<String>) -> StreamWrapper;
}

struct OperationInboxImpl<B: OperationBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: OperationBe> Default for OperationInboxImpl<B> {
    fn default() -> Self {
        return OperationInboxImpl {
            _b: std::marker::PhantomData::default(),
        };
    }
}

impl<B: OperationBe + 'static> OperationInbox for OperationInboxImpl<B> where <B::Options as Validates>::Target: Send + Sync {
    fn parse(&self, args: &mut Vec<String>) -> StreamWrapper {
        let mut opt = OptParser::<B::Options>::default();
        B::options(&mut opt.view());
        let o = opt.parse(args).validate();
        let o = Arc::new(o);
        *args = B::get_extra(o.clone());

        return StreamWrapper::new(move || B::stream(o.clone()));
    }
}

pub struct OperationRegistrant<B: OperationBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: OperationBe + 'static> Registrant<BoxedOperation> for OperationRegistrant<B> where <B::Options as Validates>::Target: Send + Sync {
    type Args = ZeroArgs;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn init2(_a: ()) -> BoxedOperation {
        return Box::new(OperationInboxImpl::<B>::default());
    }
}




pub trait OperationBe2 {
    type Options: Validates + Default + 'static;

    fn names() -> Vec<&'static str>;
    fn options<'a>(opt: &mut OptParserView<'a, Self::Options>);
    fn stream(o: Arc<<Self::Options as Validates>::Target>) -> Stream;
}

#[derive(Default)]
#[derive(Validates)]
pub struct AndArgsOptions<P: Validates> {
    p: IntoArcOption<P>,
    args: StringVecOption,
}

pub struct OperationBeForBe2<B: OperationBe2> {
    _b: std::marker::PhantomData<B>,
}

impl<B: OperationBe2> OperationBe for OperationBeForBe2<B> {
    type Options = AndArgsOptions<B::Options>;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn options<'a>(opt: &mut OptParserView<'a, AndArgsOptions<B::Options>>) {
        B::options(&mut opt.sub(|p| &mut p.p.0));
        opt.sub(|p| &mut p.args).match_extra_soft(StringVecOption::maybe_push);
    }

    fn get_extra(p: Arc<AndArgsOptionsValidated<B::Options>>) -> Vec<String> {
        return p.args.clone();
    }

    fn stream(p: Arc<AndArgsOptionsValidated<B::Options>>) -> Stream {
        return B::stream(p.p.clone());
    }
}
