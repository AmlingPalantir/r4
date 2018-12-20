use stream::Stream;

use Operation;
use record::FromPrimitive;
use record::Record;
use stream::Entry;

pub struct TestOperation {
}

impl TestOperation {
    pub fn new() -> Self {
        return TestOperation {
        };
    }
}

struct TestOperationStream {
    os: Box<Stream>,
    n: u32,
}

impl Stream for TestOperationStream {
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

impl Operation for TestOperation {
    fn wrap(&self, os: Box<Stream>) -> Box<Stream> {
        return Box::new(TestOperationStream {
            os: os,
            n: 0,
        });
    }
}
