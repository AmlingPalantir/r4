extern crate operation;
extern crate stream;
extern crate stream_process;
extern crate stream_stdout;

use operation::Operation;
use operation::test::TestOperation;
use std::io::BufRead;
use std::io;
use std::sync::Arc;
use stream::Entry;
use stream_stdout::StdoutStream;

fn main() {
    let os = Box::new(StdoutStream::new());
    let op = TestOperation::new();
    let mut os = op.wrap(os);
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if !os.write(Entry::Line(Arc::from(line))) {
            break;
        }
    }
    os.close();
}
