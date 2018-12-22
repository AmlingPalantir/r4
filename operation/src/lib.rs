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
    //aggregate,
    //bg,
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
    type PreOptions: Default + Validates<To = Self::PostOptions>;
    type PostOptions: Send + Sync + 'static;

    fn options<'a>(&'a mut OptParserView<'a, Self::PreOptions, Self::PreOptions>);
    fn wrap_stream(&Self::PostOptions, Stream) -> Stream;
}

impl<B: OperationBe> OperationFe for B {
    fn validate(&self, args: &mut Vec<String>) -> StreamWrapper {
        let o = B::PreOptions::default();
        let mut opt = OptParser::new();
        B::options(&mut opt.view());
        opt.parse(args);
        let o = o.validate();

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
