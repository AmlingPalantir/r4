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
    let op = operation::REGISTRY.find(&args.next().unwrap(), &[]);
    let mut args = args.collect();
    let op = op(&mut args);

    let mut w = |e| {
        return match e {
            Entry::Bof(_file) => true,
            Entry::Record(r) => writeln!(io::stdout(), "{}", r.deparse()).is_ok(),
            Entry::Line(line) => writeln!(io::stdout(), "{}", line).is_ok(),
        };
    };
    let mut os = op.stream();

    if args.is_empty() {
        os.write(Entry::Bof(Arc::from("-")), &mut w);
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
            os.write(Entry::Bof(Arc::from(&*arg)), &mut w);
            for line in BufReader::new(File::open(arg).unwrap()).lines() {
                if !os.write(Entry::Line(Arc::from(line.unwrap())), &mut w) {
                    break 'arg;
                }
            }
        }
    }

    os.close(&mut w);
}
