extern crate operation;
extern crate stream;
extern crate stream_process;
extern crate stream_stdout;

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
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

    let os = Stream::new(StdoutStream::new());
    let mut os = op.wrap(os);

    if args.is_empty() {
        os.bof("-");
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            os.write(Entry::Line(Arc::from(line)));
            if os.rclosed() {
                break;
            }
        }
    }
    else {
        'arg: for arg in args {
            os.bof(&arg);
            for line in BufReader::new(File::open(arg).unwrap()).lines() {
                os.write(Entry::Line(Arc::from(line.unwrap())));
                if os.rclosed() {
                    break 'arg;
                }
            }
        }
    }

    os.close();
}
