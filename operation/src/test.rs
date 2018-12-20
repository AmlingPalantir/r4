use stream::Stream;

use ClosureStreamWrapper;
use Operation;
use StreamWrapper;
use record::FromPrimitive;
use record::Record;
use stream::Entry;

struct MyOperationStream {
    os: Box<Stream>,
    n: u32,
}

impl Stream for MyOperationStream {
    fn write(&mut self, e: Entry) -> bool {
        let mut r = e.to_record();

        self.n += 1;
        r.set_path("n", Record::from_primitive(self.n));

        return self.os.write(Entry::Record(r));
    }

    fn close(&mut self) {
        self.os.close();
    }
}

pub(crate) fn name() -> &'static str {
    return "test";
}

pub(crate) fn new() -> Box<Operation> {
    return Box::new(MyOperation {
    });
}

pub struct MyOperation {
}

impl Operation for MyOperation {
    fn configure(&mut self, args: Vec<String>) -> Vec<String> {
        return args;
    }

    fn validate(&self) -> Box<StreamWrapper> {
        return ClosureStreamWrapper::new(|os| {
            return Box::new(MyOperationStream {
                os: os,
                n: 0,
            });
        });
    }
}
