extern crate stream;

use std::io::Write;
use std::io;
use stream::Entry;
use stream::StreamTrait;

pub struct StdoutStream {
    rclosed: bool,
}

impl StdoutStream {
    pub fn new() -> Self {
        return StdoutStream {
            rclosed: false,
        };
    }

    fn maybe_rclosed<T, E>(&mut self, r: Result<T, E>) {
        match r {
            Err(_) => {
                self.rclosed = true;
            }
            Ok(_) => {
            }
        }
    }
}

impl StreamTrait for StdoutStream {
    fn write(&mut self, e: Entry) {
        match e {
            Entry::Bof(_file) => {
            }
            Entry::Record(r) => {
                self.maybe_rclosed(writeln!(io::stdout(), "{}", r.to_string()));
            }
            Entry::Line(line) => {
                self.maybe_rclosed(writeln!(io::stdout(), "{}", line));
            }
            Entry::Close() => {
                // This seems to be all we can do?  We hope/expect the process
                // to be donezo soon anyway...
                self.maybe_rclosed(io::stdout().flush());
            }
        }
    }

    fn rclosed(&mut self) -> bool {
        return self.rclosed;
    }
}
