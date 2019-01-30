extern crate operation;
extern crate stream;

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::io;
use std::sync::Arc;
use stream::Entry;

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let op = args.next().unwrap_or_else(|| "help".to_string());
    let op = operation::REGISTRY.find(&op, &[]).unwrap();
    let mut args = args.collect();
    let op = op.parse(&mut args);

    let mut w = |e: Entry| {
        return writeln!(io::stdout(), "{}", e.deparse()).is_ok();
    };
    let mut os = op.stream();

    if args.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            if !os.write(Entry::Line(Arc::from(line)), &mut w) {
                break;
            }
        }
    }
    else {
        'arg: for arg in args {
            for line in BufReader::new(File::open(arg).unwrap()).lines() {
                if !os.write(Entry::Line(Arc::from(line.unwrap())), &mut w) {
                    break 'arg;
                }
            }
        }
    }

    os.close(&mut w);
}
