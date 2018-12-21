extern crate bgop;
extern crate stream;

use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use stream::Entry;
use stream::Stream;
use stream::StreamTrait;

pub struct ProcessStream {
    os: Stream,
    p: Child,
}

impl ProcessStream {
    pub fn new<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(os: Stream, args: I) -> Self {
        let mut args = args.into_iter();
        let mut p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let (fe, rbe, mut wbe) = bgop::new(os);
        let p_stdin = p.stdin.take().unwrap();
        let p_stdout = p.stdout.take().unwrap();

        thread::spawn(move || {
            let mut lw = LineWriter::new(p_stdin);
            loop {
                match rbe.read() {
                    Entry::Bof(_file) => {
                        continue;
                    }
                    Entry::Record(r) => {
                        if let Err(_) = writeln!(lw, "{}", r.to_string()) {
                            rbe.rclose();
                        }
                    }
                    Entry::Line(line) => {
                        if let Err(_) = writeln!(lw, "{}", line) {
                            rbe.rclose();
                        }
                    }
                    Entry::Close() => {
                        // drops r
                        return;
                    }
                }
            }
        });

        thread::spawn(move || {
            let r = BufReader::new(p_stdout);
            for line in r.lines() {
                let line = line.unwrap();
                wbe.write(Entry::Line(Arc::from(line)));
                if wbe.rclosed() {
                    break;
                }
            }
            wbe.write(Entry::Close());
            // return drops r
        });

        return ProcessStream {
            os: Stream::new(fe),
            p: p,
        };
    }
}

impl StreamTrait for ProcessStream {
    fn write(&mut self, e: Entry) {
        let close = e.is_close();
        self.os.write(e);
        if close {
            self.p.wait().unwrap();
        }
    }

    fn rclosed(&mut self) -> bool {
        return self.os.rclosed();
    }
}
