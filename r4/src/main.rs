extern crate operation;
extern crate stream;
extern crate stream_process;
extern crate stream_stdout;

use std::env;
use std::io::BufRead;
use std::io;
use std::sync::Arc;
use stream::Entry;
use stream_stdout::StdoutStream;

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let mut op = operation::find_operation(&args.next().unwrap());
    let args = op.configure(args.collect());
    assert!(args.is_empty());
    let op = op.validate();

    let os = Box::new(StdoutStream::new());
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
