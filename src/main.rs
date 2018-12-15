use std::collections::VecDeque;
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
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;

trait Stream {
    fn write_line(&mut self, String);
    fn rclosed(&mut self) -> bool;
    fn close(&mut self);
}

fn main() {
    let os = StdoutStream::new();
    let mut os = ProcessStream::new(Box::new(os), env::args().skip(1));
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        println!("Input line: {}", line);
        os.write_line(line);
        if os.rclosed() {
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
    fn write_line(&mut self, line: String) {
        println!("StdoutStream line: {}", line);
    }

    fn rclosed(&mut self) -> bool {
        return false;
    }

    fn close(&mut self) {
    }
}

struct ProcessBuffer {
    lines: VecDeque<Option<String>>,
    rclosed: bool,
}

impl ProcessBuffer {
    fn new() -> ProcessBuffer {
        return ProcessBuffer {
            lines: VecDeque::new(),
            rclosed: false,
        };
    }
}

struct ProcessBuffers {
    os_closed: bool,
    stdin: ProcessBuffer,
    stdout: ProcessBuffer,
}

impl ProcessBuffers {
    fn new() -> ProcessBuffers {
        return ProcessBuffers {
            os_closed: false,
            stdin: ProcessBuffer::new(),
            stdout: ProcessBuffer::new(),
        };
    }
}

struct ProcessStream {
    os: Box<Stream>,
    p: Child,
    buffers: Arc<(Condvar, Mutex<ProcessBuffers>)>,
}

impl ProcessStream {
    fn new<I, S>(os: Box<Stream>, args: I) -> ProcessStream where I: IntoIterator<Item = S>, S: AsRef<OsStr> {
        let mut args = args.into_iter();
        let p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let buffers = Arc::new((Condvar::new(), Mutex::new(ProcessBuffers::new())));
        {
            let p_stdin = p.stdin;
            let buffers = buffers.clone();
            thread::spawn(move|| {
                let mut r = LineWriter::new(p_stdin.unwrap());
                let (ref cond, ref buffers) = *buffers;
                loop {
                    fn read_line(cond: &Condvar, buffers: &Mutex<ProcessBuffers>) -> Option<String> {
                        let mut buffers = buffers.lock().unwrap();
                        loop {
                            while let Some(maybe_line) = buffers.stdin.lines.pop_front() {
                                cond.notify_all();
                                return maybe_line;
                            }
                            buffers = cond.wait(buffers).unwrap();
                        }
                    }
                    match read_line(cond, buffers) {
                        Some(line) => {
                            println!("[backend stdin] got line {}", line);
                            let mut bytes = line.into_bytes();
                            bytes.push(b'\n');
                            match r.write_all(&bytes) {
                                Err(_) => {
                                    let mut buffers = buffers.lock().unwrap();
                                    buffers.stdin.rclosed = true;
                                    buffers.stdin.lines.clear();
                                    cond.notify_all();
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
            let p_stdout = p.stdout;
            let buffers = buffers.clone();
            thread::spawn(move|| {
                let r = BufReader::new(p_stdout.unwrap());
                let (ref cond, ref buffers) = *buffers;
                for line in r.lines() {
                    let line = line.unwrap();
                    let mut buffers = buffers.lock().unwrap();
                    loop {
                        if buffers.stdout.rclosed {
                            // drops r
                            return;
                        }
                        if buffers.stdout.lines.len() < 1024 {
                            buffers.stdout.lines.push_back(Some(line));
                            cond.notify_all();
                            break;
                        }
                        buffers = cond.wait(buffers).unwrap();
                    }
                }
                {
                    let mut buffers = buffers.lock().unwrap();
                    buffers.stdout.lines.push_back(None);
                    cond.notify_all();
                }
            });
        }

        return ProcessStream {
            os: os,
            p: p,
            buffers: buffers,
        };
    }
}

impl Stream for ProcessStream {
    fn write_line(&mut self, line: String) {
        let (ref cond, ref buffers) = *self.buffers;
        let mut buffers = buffers.lock().unwrap();
        loop {
            while let Some(maybe_line) = buffers.stdout.lines.pop_front() {
                cond.notify_all();
                match maybe_line {
                    Some(line) => {
                        println!("[line ferry] Output line: {}", line);
                        self.os.write_line(line);
                    }
                    None => {
                        self.os.close();
                        buffers.os_closed = true
                    }
                }
            }

            if buffers.stdin.rclosed {
                println!("[frontend] input dropped");
                return;
            }
            if buffers.stdin.lines.len() < 1024 {
                println!("[frontend] input ready");
                buffers.stdin.lines.push_back(Some(line));
                cond.notify_all();
                return;
            }

            buffers = cond.wait(buffers).unwrap();
        }
    }

    fn rclosed(&mut self) -> bool {
        let (_, ref buffers) = *self.buffers;
        let buffers = buffers.lock().unwrap();
        return buffers.stdin.rclosed;
    }

    fn close(&mut self) {
        let (ref cond, ref buffers) = *self.buffers;
        let mut buffers = buffers.lock().unwrap();
        buffers.stdin.lines.push_back(None);
        loop {
            while let Some(maybe_line) = buffers.stdout.lines.pop_front() {
                cond.notify_all();
                match maybe_line {
                    Some(line) => {
                        println!("[line ferry] Output line: {}", line);
                        self.os.write_line(line);
                    }
                    None => {
                        self.os.close();
                        buffers.os_closed = true
                    }
                }
            }

            if buffers.os_closed {
                break;
            }

            buffers = cond.wait(buffers).unwrap();
        }

        self.p.wait().unwrap();
    }
}
