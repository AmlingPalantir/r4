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
use stream::Line;
use stream::Stream;

pub struct ProcessStream {
    p: Child,
    bgop_fe: BgopFe<Line>,
}

impl ProcessStream {
    pub fn new<I, S>(mut os: Box<Stream>, args: I) -> ProcessStream where I: IntoIterator<Item = S>, S: AsRef<OsStr> {
        let mut args = args.into_iter();
        let mut p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let bgop_fe = BgopFe::new(Box::new(move |maybe_line| {
            match maybe_line {
                Some(line) => {
                    return os.write_line(line);
                }
                None => {
                    os.close();
                    return false;
                }
            }
        }));
        {
            let p_stdin = p.stdin.take().unwrap();
            let bgop_be = bgop_fe.be();
            thread::spawn(move|| {
                let mut r = LineWriter::new(p_stdin);
                loop {
                    match bgop_be.read_line() {
                        Some(line) => {
                            eprintln!("[backend stdin] got line {}", line);
                            match writeln!(r, "{}", line) {
                                Err(_) => {
                                    eprintln!("[backend stdin] got rclosed");
                                    bgop_be.rclose();
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
            let bgop_be = bgop_fe.be();
            thread::spawn(move|| {
                let r = BufReader::new(p_stdout);
                for line in r.lines() {
                    let line = line.unwrap();
                    if !bgop_be.write_line(Arc::from(line)) {
                        eprintln!("[backend stdout] got rclosed");
                        break;
                    }
                }
                bgop_be.close();
                // return drops r
            });
        }

        return ProcessStream {
            p: p,
            bgop_fe: bgop_fe,
        };
    }
}

impl Stream for ProcessStream {
    fn write_line(&mut self, line: Line) -> bool {
        return self.bgop_fe.write_line(line);
    }

    fn close(&mut self) {
        self.bgop_fe.close();
        self.p.wait().unwrap();
    }
}
