extern crate wns;

mod bgop;
mod stream;

use std::env;
use std::io::BufRead;
use std::io;
use std::sync::Arc;
use stream::Stream;
use stream::process::ProcessStream;
use stream::stdout::StdoutStream;

fn main() {
    let os = StdoutStream::new();
    let mut os = ProcessStream::new(Box::new(os), env::args().skip(1));
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        eprintln!("[main] Input line: {}", line);
        os.write_line(Arc::from(line));
        if os.rclosed() {
            eprintln!("[main] got rclosed");
            break;
        }
    }
    os.close();
}
