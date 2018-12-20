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

pub struct ProcessStream {
    p: Child,
    bgop: BgopFe<Arc<str>>,
}

impl ProcessStream {
    pub fn new<OS: Stream + 'static, I: IntoIterator<Item = S>, S: AsRef<OsStr>>(os: OS, args: I) -> Self {
        return Self::new_box(Box::new(os), args);
    }

    pub fn new_box<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(mut os: Box<Stream>, args: I) -> Self {
        let mut args = args.into_iter();
        let mut p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let bgop = BgopFe::<Arc<str>>::new(move |maybe_line| {
            match maybe_line {
                Some(line) => {
                    return os.write(Entry::Line(line));
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
            thread::spawn(move|| {
                let mut r = LineWriter::new(p_stdin);
                loop {
                    match bgop.read_line() {
                        Some(line) => {
                            match writeln!(r, "{}", line) {
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
            let bgop = bgop.be();
            thread::spawn(move|| {
                let r = BufReader::new(p_stdout);
                for line in r.lines() {
                    let line = line.unwrap();
                    if !bgop.write(Arc::from(line)) {
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

impl Stream for ProcessStream {
    fn write(&mut self, e: Entry) -> bool {
        return self.bgop.write(e.to_line());
    }

    fn close(&mut self) {
        self.bgop.close();
        self.p.wait().unwrap();
    }
}
