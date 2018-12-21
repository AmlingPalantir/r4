extern crate bgop;
extern crate stream;

use bgop::BgopFe;
use bgop::BofOrWrite;
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

        let bgop = BgopFe::new(move |maybe_e| {
            match maybe_e {
                Some(BofOrWrite::Bof(file)) => {
                    os.bof(&file);
                    return !os.rclosed();
                }
                Some(BofOrWrite::Write(e)) => {
                    os.write(e);
                    return !os.rclosed();
                }
                None => {
                    os.close();
                    return false;
                }
            }
        });
        {
            let p_stdin = p.stdin.take().unwrap();
            let bgop = bgop.be();
            thread::spawn(move || {
                let mut r = LineWriter::new(p_stdin);
                loop {
                    match bgop.read() {
                        Some(BofOrWrite::Bof(_file)) => {
                        }
                        Some(BofOrWrite::Write(e)) => {
                            match writeln!(r, "{}", e.to_line()) {
                                Err(_) => {
                                    bgop.rclose();
                                }
                                Ok(_) => {
                                }
                            }
                        }
                        None => {
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
                bgop.close();
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
    fn bof(&mut self, file: &str) {
        self.bgop.bof(file);
    }

    fn write(&mut self, e: Entry) {
        self.bgop.write(e);
    }

    fn rclosed(&mut self) -> bool {
        return self.bgop.rclosed();
    }

    fn close(&mut self) {
        self.bgop.close();
        self.p.wait().unwrap();
    }
}
