extern crate operation;
extern crate stream;
extern crate stream_process;
extern crate stream_stdout;

use std::env;
use std::io::BufRead;
use std::io;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use stream::StreamTrait;
use stream_stdout::StdoutStream;

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let op = operation::find(&args.next().unwrap());
    let mut args = args.collect();
    let op = op.validate(&mut args);
    assert!(args.is_empty());

    let os = Stream::new(StdoutStream::new());
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
