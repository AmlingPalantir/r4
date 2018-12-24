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
    type PreOptions: Validates<To = Self::PostOptions> + Default + 'static;
    type PostOptions: Send + Sync + 'static;

    fn options<'a>(OptParserView<'a, Self::PreOptions>);
    fn get_extra(&Self::PostOptions) -> &Vec<String>;
    fn stream(&Self::PostOptions) -> Stream;
}

impl<B: OperationBe> OperationFe for B {
    fn validate(&self, args: &mut Vec<String>) -> StreamWrapper {
        let mut opt = OptParser::<B::PreOptions>::new();
        B::options(opt.view());
        let o = opt.parse(args).validate();
        *args = B::get_extra(&o).clone();

        return StreamWrapper::new(move || B::stream(&o));
    }
}



pub trait OperationBe2 {
    type PreOptions: Validates<To = Self::PostOptions> + Default + 'static;
    type PostOptions: Send + Sync + 'static;

    fn options<'a>(OptParserView<'a, Self::PreOptions>);
    fn stream(&Self::PostOptions) -> Stream;
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

    fn options<'a>(mut opt: OptParserView<'a, AndArgsOptions<B::PreOptions>>) {
        B::options(opt.sub(|p| &mut p.p));
        opt.sub(|p| &mut p.args).match_extra_soft(|p, a| {
            p.push(a.clone());
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
