use stream::Stream;

use ClosureStreamWrapper;
use Operation;
use StreamWrapper;
use record::FromPrimitive;
use record::Record;
use std::collections::VecDeque;
use std::sync::Arc;
use stream::Entry;

struct MyOperationStream {
    os: Box<Stream>,
    msg: Arc<str>,
    n: u32,
}

impl Stream for MyOperationStream {
    fn write(&mut self, e: Entry) -> bool {
        let mut r = e.to_record();

        self.n += 1;
        r.set_path("n", Record::from_primitive(self.n));
        r.set_path("msg", Record::from_primitive_string(self.msg.clone()));

        return self.os.write(Entry::Record(r));
    }

    fn close(&mut self) {
        self.os.close();
    }
}

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["test"];
}

#[derive(Default)]
pub struct Impl {
}

impl Operation for Impl {
    fn validate(&self, args: &mut VecDeque<String>) -> Box<StreamWrapper> {
        let msg: Arc<str> = Arc::from(args.pop_front().unwrap());

        return ClosureStreamWrapper::new(move |os| {
            return Box::new(MyOperationStream {
                os: os,
                msg: msg.clone(),
                n: 0,
            });
        });
    }
}
