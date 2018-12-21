extern crate bgop;
extern crate stream;

use bgop::BgopFe;
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
    p: Child,
    bgop: BgopFe,
}

impl ProcessStream {
    pub fn new<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(mut os: Stream, args: I) -> Self {
        let mut args = args.into_iter();
        let mut p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let bgop = BgopFe::new(move |e| {
            os.write(e);
            return !os.rclosed();
        });
        {
            let p_stdin = p.stdin.take().unwrap();
            let bgop = bgop.be();
            thread::spawn(move || {
                let mut lw = LineWriter::new(p_stdin);
                loop {
                    match bgop.read() {
                        Entry::Bof(_file) => {
                            continue;
                        }
                        Entry::Record(r) => {
                            if let Err(_) = writeln!(lw, "{}", r.to_string()) {
                                bgop.rclose();
                            }
                        }
                        Entry::Line(line) => {
                            if let Err(_) = writeln!(lw, "{}", line) {
                                bgop.rclose();
                            }
                        }
                        Entry::Close() => {
                            // drops r
                            return;
                        }
                    }
                }
            });
        }

        {
            let p_stdout = p.stdout.take().unwrap();
            let mut bgop = bgop.be();
            thread::spawn(move || {
                let r = BufReader::new(p_stdout);
                for line in r.lines() {
                    let line = line.unwrap();
                    bgop.write(Entry::Line(Arc::from(line)));
                    if bgop.rclosed() {
                        break;
                    }
                }
                bgop.write(Entry::Close());
                // return drops r
            });
        }

        return ProcessStream {
            p: p,
            bgop: bgop,
        };
    }
}

impl StreamTrait for ProcessStream {
    fn write(&mut self, e: Entry) {
        let close = e.is_close();
        self.bgop.write(e);
        if close {
            self.p.wait().unwrap();
        }
    }

    fn rclosed(&mut self) -> bool {
        return self.bgop.rclosed();
    }
}
