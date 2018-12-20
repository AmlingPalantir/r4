extern crate stream;
extern crate stream_process;
extern crate stream_stdout;

use std::env;
use std::io::BufRead;
use std::io;
use std::sync::Arc;
use stream::Stream;
use stream_process::ProcessStream;
use stream_stdout::StdoutStream;

fn main() {
    let os = StdoutStream::new();
    let mut os = ProcessStream::new(os, env::args().skip(1));
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if !os.write_line(Arc::from(line)) {
            break;
        }
    }
    os.close();
}
