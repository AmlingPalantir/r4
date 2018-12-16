mod bgop;
mod wns;

use bgop::BackgroundOp;
use std::env;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::io;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;

trait Stream {
    fn write_line(&mut self, Arc<str>);
    fn rclosed(&mut self) -> bool;
    fn close(&mut self);
}

fn main() {
    let os = StdoutStream::new();
    let mut os = ProcessStream::new(Box::new(os), env::args().skip(1));
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        println!("[main] Input line: {}", line);
        os.write_line(Arc::from(line));
        if os.rclosed() {
            println!("[main] got rclosed");
            break;
        }
    }
    os.close();
}

struct StdoutStream {
}

impl StdoutStream {
    fn new() -> StdoutStream {
        return StdoutStream {
        };
    }
}

impl Stream for StdoutStream {
    fn write_line(&mut self, line: Arc<str>) {
        println!("StdoutStream line: {}", line);
    }

    fn rclosed(&mut self) -> bool {
        return false;
    }

    fn close(&mut self) {
    }
}

struct ProcessStream {
    os: Box<Stream>,
    p: Child,
    os_closed: bool,
    bgop: BackgroundOp<Arc<str>>,
}

impl ProcessStream {
    fn new<I, S>(os: Box<Stream>, args: I) -> ProcessStream where I: IntoIterator<Item = S>, S: AsRef<OsStr> {
        let mut args = args.into_iter();
        let mut p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let bgop = BackgroundOp::<Arc<str>>::new();
        {
            let p_stdin = p.stdin.take().unwrap();
            let bgop = bgop.clone();
            thread::spawn(move|| {
                let mut r = LineWriter::new(p_stdin);
                loop {
                    match bgop.be_read_line() {
                        Some(line) => {
                            println!("[backend stdin] got line {}", line);
                            let mut bytes = Vec::new();
                            bytes.extend_from_slice(line.as_bytes());
                            bytes.push(b'\n');
                            match r.write_all(&bytes) {
                                Err(_) => {
                                    println!("[backend stdin] got rclosed");
                                    bgop.be_rclose();
                                }
                                Ok(_) => {
                                }
                            }
                        }
                        None => {
                            println!("[backend stdin] got eof");
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
                        println!("[backend stdout] got rclosed");
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
            os_closed: false,
            bgop: bgop,
        };
    }
}

impl ProcessStream {
    fn write_on_maybe_line(&self, maybe_line: Option<Arc<str>>) {
        match maybe_line {
            Some(line) => {
                self.os.write_line(line);
                if self.os.rclosed() {
                    self.bgop.fe_rclose();
                }
            }
            None => {
                self.os.close();
                self.os_closed = true;
            }
        }
    }
}

impl Stream for ProcessStream {
    fn write_line(&mut self, line: Arc<str>) {
        self.bgop.fe_write_line(line, |x| self.write_on_maybe_line(x));
    }

    fn rclosed(&mut self) -> bool {
        return self.bgop.fe_rclosed();
    }

    fn close(&mut self) {
        self.bgop.fe_close(|x| self.write_on_maybe_line(x));
        self.p.wait().unwrap();
    }
}
