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
    bgop: Arc<BackgroundOp<Line>>,
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

fn write_on_maybe_line(os: &mut Box<Stream>, bgop: &BackgroundOp<Line>, maybe_line: Option<Line>) {
    match maybe_line {
        Some(line) => {
            os.write_line(line);
            if os.rclosed() {
                bgop.fe_rclose();
            }
        }
        None => {
            os.close();
        }
    }
}

impl Stream for ProcessStream {
    fn write_line(&mut self, line: Line) {
        let os = &mut self.os;
        let bgop = &self.bgop;
        self.bgop.fe_write_line(line, &mut |x| write_on_maybe_line(os, bgop, x));
    }

    fn rclosed(&mut self) -> bool {
        return self.bgop.fe_rclosed();
    }

    fn close(&mut self) {
        let os = &mut self.os;
        let bgop = &self.bgop;
        self.bgop.fe_close(&mut |x| write_on_maybe_line(os, bgop, x));
        self.p.wait().unwrap();
    }
}
