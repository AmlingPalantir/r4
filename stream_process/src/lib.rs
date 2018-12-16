extern crate bgop;
extern crate stream;

use bgop::BackgroundOp;
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
use stream::Line;
use stream::Stream;

pub struct ProcessStream {
    os: Box<Stream>,
    p: Child,
    bgop: Arc<BackgroundOp<Line, _>>,
}

impl ProcessStream {
    pub fn new<I, S>(os: Box<Stream>, args: I) -> ProcessStream where I: IntoIterator<Item = S>, S: AsRef<OsStr> {
        let mut args = args.into_iter();
        let mut p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let bgop = Arc::new(BackgroundOp::<Line>::new());
        {
            let p_stdin = p.stdin.take().unwrap();
            let bgop = bgop.clone();
            thread::spawn(move|| {
                let mut r = LineWriter::new(p_stdin);
                loop {
                    match bgop.be_read_line() {
                        Some(line) => {
                            eprintln!("[backend stdin] got line {}", line);
                            match writeln!(r, "{}", line) {
                                Err(_) => {
                                    eprintln!("[backend stdin] got rclosed");
                                    bgop.be_rclose();
                                }
                                Ok(_) => {
                                }
                            }
                        }
                        None => {
                            eprintln!("[backend stdin] got eof");
                            // drops r
                            return;
                        }
                    }
                }
            });
        }

        {
            let p_stdout = p.stdout.take().unwrap();
            let bgop = bgop.clone();
            thread::spawn(move|| {
                let r = BufReader::new(p_stdout);
                for line in r.lines() {
                    let line = line.unwrap();
                    if !bgop.be_write_line(Arc::from(line)) {
                        eprintln!("[backend stdout] got rclosed");
                        break;
                    }
                }
                bgop.be_close();
                // return drops r
            });
        }

        return ProcessStream {
            os: os,
            p: p,
            bgop: bgop,
        };
    }
}

fn write_on_maybe_line(os: &mut Box<Stream>, maybe_line: Option<Line>) -> bool {
    match maybe_line {
        Some(line) => {
            return os.write_line(line);
        }
        None => {
            os.close();
            return false;
        }
    }
}

impl Stream for ProcessStream {
    fn write_line(&mut self, line: Line) -> bool {
        let os = &mut self.os;
        return self.bgop.fe_write_line(line, &mut |x| write_on_maybe_line(os, x));
    }

    fn close(&mut self) {
        let os = &mut self.os;
        self.bgop.fe_close(&mut |x| write_on_maybe_line(os, x));
        self.p.wait().unwrap();
    }
}
