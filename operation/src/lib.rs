extern crate aggregator;
extern crate bgop;
#[macro_use]
extern crate opts;
extern crate record;
#[macro_use]
extern crate registry;
extern crate stream;

registry! {
    OperationFe:
    aggregate,
    bg,
    test,
}

use opts::OptParser;
use opts::OptParserView;
use opts::Validates;
use stream::Stream;

pub trait OperationFe {
    fn validate(&self, &mut Vec<String>) -> StreamWrapper;
}

pub trait OperationBe {
    type PreOptions: Default + Validates<To = Self::PostOptions> + 'static;
    type PostOptions: Send + Sync + 'static;

    fn options<'a, X: 'static>(&'a mut OptParserView<'a, X, Self::PreOptions>);
    fn wrap_stream(&Self::PostOptions, Stream) -> Stream;
}

#[derive(Default)]
struct FeOptions<P> {
    p: P,
    args: Vec<String>,
}

impl<B: OperationBe> OperationFe for B {
    fn validate(&self, args: &mut Vec<String>) -> StreamWrapper {
        let mut opt = OptParser::<FeOptions<B::PreOptions>>::new();
        B::options(&mut opt.view().sub(|p| &mut p.p));
        opt.view().sub(|p| &mut p.args).match_extra_soft(|p, a| {
            p.push(a.clone());
            return true;
        });
        let p = opt.parse(args);
        *args = p.args;
        let o = p.p.validate();

        return StreamWrapper::new(move |os| B::wrap_stream(&o, os));
    }
}

pub struct StreamWrapper(Box<Fn(Stream) -> Stream + Send + Sync>);

impl StreamWrapper {
    pub fn new<F: Fn(Stream) -> Stream + 'static + Send + Sync>(f: F) -> Self {
        return StreamWrapper(Box::new(f));
    }

    pub fn wrap(&self, os: Stream) -> Stream {
        return self.0(os);
    }
}
