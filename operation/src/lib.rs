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

pub struct StreamWrapper(Box<Fn(Stream) -> Stream + Send + Sync>);

impl StreamWrapper {
    pub fn new<F: Fn(Stream) -> Stream + 'static + Send + Sync>(f: F) -> Self {
        return StreamWrapper(Box::new(f));
    }

    pub fn wrap(&self, os: Stream) -> Stream {
        return self.0(os);
    }
}



pub trait OperationBe {
    type PreOptions: Default + Validates<To = Self::PostOptions> + 'static;
    type PostOptions: Send + Sync + 'static;

    fn options<'a, X: 'static>(&'a mut OptParserView<'a, X, Self::PreOptions>);
    fn get_extra(&Self::PostOptions) -> &Vec<String>;
    fn wrap_stream(&Self::PostOptions, Stream) -> Stream;
}

impl<B: OperationBe> OperationFe for B {
    fn validate(&self, args: &mut Vec<String>) -> StreamWrapper {
        let mut opt = OptParser::<B::PreOptions>::new();
        B::options(&mut opt.view());
        let o = opt.parse(args).validate();
        *args = B::get_extra(&o).clone();

        return StreamWrapper::new(move |os| B::wrap_stream(&o, os));
    }
}



pub trait OperationBe2 {
    type PreOptions: Default + Validates<To = Self::PostOptions> + 'static;
    type PostOptions: Send + Sync + 'static;

    fn options<'a, X: 'static>(&'a mut OptParserView<'a, X, Self::PreOptions>);
    fn wrap_stream(&Self::PostOptions, Stream) -> Stream;
}

#[derive(Default)]
pub struct AndArgsOptions<P> {
    p: P,
    args: Vec<String>,
}

impl<V, P: Validates<To = V>> Validates for AndArgsOptions<P> {
    type To = AndArgsOptions<V>;

    fn validate(self) -> AndArgsOptions<V> {
        return AndArgsOptions {
            p: self.p.validate(),
            args: self.args,
        };
    }
}

impl<B: OperationBe2> OperationBe for B {
    type PreOptions = AndArgsOptions<B::PreOptions>;
    type PostOptions = AndArgsOptions<B::PostOptions>;

    fn options<'a, X: 'static>(opt: &'a mut OptParserView<'a, X, AndArgsOptions<B::PreOptions>>) {
        B::options(&mut opt.sub(|p| &mut p.p));
        opt.sub(|p| &mut p.args).match_extra_soft(|p, a| {
            p.push(a.clone());
            return true;
        });
    }

    fn get_extra(p: &AndArgsOptions<B::PostOptions>) -> &Vec<String> {
        return &p.args;
    }

    fn wrap_stream(p: &AndArgsOptions<B::PostOptions>, os: Stream) -> Stream {
        return B::wrap_stream(&p.p, os);
    }
}
