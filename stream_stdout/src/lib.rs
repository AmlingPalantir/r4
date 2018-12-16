extern crate stream;

use std::io::Write;
use std::io;
use stream::Line;
use stream::Stream;

pub struct StdoutStream {
    rclosed: bool,
}

impl StdoutStream {
    pub fn new() -> StdoutStream {
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

impl Stream for StdoutStream {
    fn write_line(&mut self, line: Line) {
        self.maybe_rclosed(writeln!(io::stdout(), "{}", line));
    }

    fn rclosed(&mut self) -> bool {
        return self.rclosed;
    }

    fn close(&mut self) {
        // This seems to be all we can do?  We hope/expect the process to be
        // donezo soon anyway...
        self.maybe_rclosed(io::stdout().flush());
    }
}
