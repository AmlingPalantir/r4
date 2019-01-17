#[macro_use]
extern crate lalrpop_util;
#[macro_use]
extern crate lazy_static;
extern crate misc;
extern crate record;
#[macro_use]
extern crate registry;
extern crate rlua;
extern crate validates;

use record::Record;
use registry::Registrant;
use registry::args::ZeroArgs;
use validates::ValidationResult;

pub type BoxedExecutor = Box<ExecutorInbox>;
pub type BoxedExecutor2 = Box<Executor2Inbox>;

registry! {
    BoxedExecutor,
    lua,
    r4l,
}

pub trait ExecutorInbox {
    fn parse(&self, code: &str) -> ValidationResult<BoxedExecutor2>;
}

pub trait Executor2Inbox: Send + Sync {
    fn stream(&self, ret: bool) -> Box<FnMut(Record) -> Record>;
    fn box_clone(&self) -> BoxedExecutor2;
}

impl Clone for BoxedExecutor2 {
    fn clone(&self) -> BoxedExecutor2 {
        return self.box_clone();
    }
}

pub trait ExecutorBe {
    type Code: Clone + Send + Sync;

    fn names() -> Vec<&'static str>;
    fn parse(code: &str) -> ValidationResult<Self::Code>;
    fn stream(code: &Self::Code, ret: bool) -> Box<FnMut(Record) -> Record>;
}

pub struct ExecutorRegistrant<B: ExecutorBe> {
    _b: std::marker::PhantomData<B>,
}

struct ExecutorInboxImpl<B: ExecutorBe> {
    _b: std::marker::PhantomData<B>,
}

struct Executor2InboxImpl<B: ExecutorBe> {
    code: <B as ExecutorBe>::Code,
}

impl<B: ExecutorBe + 'static> Registrant<BoxedExecutor> for ExecutorRegistrant<B> {
    type Args = ZeroArgs;

    fn names() -> Vec<&'static str> {
        return <B as ExecutorBe>::names();
    }

    fn init2(_a: ()) -> BoxedExecutor {
        return Box::new(ExecutorInboxImpl {
            _b: std::marker::PhantomData::<B>,
        });
    }
}

impl<B: ExecutorBe + 'static> ExecutorInbox for ExecutorInboxImpl<B> {
    fn parse(&self, code: &str) -> ValidationResult<BoxedExecutor2> {
        return Result::Ok(Box::new(Executor2InboxImpl::<B> {
            code: B::parse(code)?,
        }));
    }
}

impl<B: ExecutorBe + 'static> Executor2Inbox for Executor2InboxImpl<B> {
    fn stream(&self, ret: bool) -> Box<FnMut(Record) -> Record> {
        return <B as ExecutorBe>::stream(&self.code, ret);
    }

    fn box_clone(&self) -> BoxedExecutor2 {
        return Box::new(Executor2InboxImpl::<B> {
            code: self.code.clone(),
        });
    }
}
