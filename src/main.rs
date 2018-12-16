mod wns;

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
use std::thread;
use wns::WaitNotifyState;

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

#[derive(Clone)]
struct ProcessBuffer {
    lines: VecDeque<Option<Arc<str>>>,
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

#[derive(Clone)]
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
    buffers: WaitNotifyState<ProcessBuffers>,
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

        let buffers = WaitNotifyState::new(ProcessBuffers::new());
        {
            let p_stdin = p.stdin.take().unwrap();
            let buffers = buffers.clone();
            thread::spawn(move|| {
                let mut r = LineWriter::new(p_stdin);
                loop {
                    let maybe_line = buffers.await(|buffers| {
                        if let Some(maybe_line) = buffers.stdin.lines.pop_front() {
                            return (Some(maybe_line), true);
                        }
                        return (None, false);
                    });
                    match maybe_line {
                        Some(line) => {
                            println!("[backend stdin] got line {}", line);
                            let mut bytes = Vec::new();
                            bytes.extend_from_slice(line.as_bytes());
                            bytes.push(b'\n');
                            match r.write_all(&bytes) {
                                Err(_) => {
                                    println!("[backend stdin] got rclosed");
                                    buffers.write(|buffers| {
                                        buffers.stdin.rclosed = true;
                                        buffers.stdin.lines.clear();
                                    });
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
            let buffers = buffers.clone();
            thread::spawn(move|| {
                let r = BufReader::new(p_stdout);
                'LINE: for line in r.lines() {
                    let line = line.unwrap();
                    let cont = buffers.await(|buffers| {
                        if buffers.stdout.rclosed {
                            println!("[backend stdout] got rclosed");
                            return (Some(false), false);
                        }
                        if buffers.stdout.lines.len() < 1024 {
                            buffers.stdout.lines.push_back(Some(Arc::from(line.clone())));
                            return (Some(true), true);
                        }
                        return (None, false);
                    });
                    if !cont {
                        break 'LINE;
                    }
                }
                buffers.write(|buffers| {
                    buffers.stdout.lines.push_back(None);
                });
                // return drops r
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
    fn write_line(&mut self, line: Arc<str>) {
        loop {
            let ret = self.buffers.await(|buffers| {
                if buffers.stdout.lines.len() > 0 {
                    let mut ret = Vec::new();
                    while let Some(maybe_line) = buffers.stdout.lines.pop_front() {
                        ret.push(maybe_line);
                    }
                    return (Some(Some(ret)), true);
                }

                if buffers.stdin.rclosed {
                    println!("[frontend] input dropped");
                    return (Some(None), false);
                }

                if buffers.stdin.lines.len() < 1024 {
                    println!("[frontend] input ready");
                    buffers.stdin.lines.push_back(Some(line.clone()));
                    return (Some(None), true);
                }

                return (None, false);
            });
            match ret {
                Some(maybe_lines) => {
                    for maybe_line in maybe_lines {
                        match maybe_line {
                            Some(line) => {
                                println!("[line ferry] Output line: {}", line);
                                self.os.write_line(line);
                                if self.os.rclosed() {
                                    println!("[line ferry] got rclosed");
                                    self.buffers.write(|buffers| {
                                        buffers.stdout.rclosed = true;
                                        buffers.stdout.lines.clear();
                                    });
                                }
                            }
                            None => {
                                self.os.close();
                                self.buffers.write(|buffers| {
                                    buffers.os_closed = true;
                                });
                            }
                        }
                    }
                }
                None => {
                    return;
                }
            }
        }
    }

    fn rclosed(&mut self) -> bool {
        return self.buffers.read(|buffers| {
            return buffers.stdin.rclosed;
        });
    }

    fn close(&mut self) {
        self.buffers.write(|buffers| {
            buffers.stdin.lines.push_back(None);
        });
        loop {
            let ret = self.buffers.await(|buffers| {
                if buffers.stdout.lines.len() > 0 {
                    let mut ret = Vec::new();
                    while let Some(maybe_line) = buffers.stdout.lines.pop_front() {
                        ret.push(maybe_line);
                    }
                    return (Some(Some(ret)), true);
                }

                if buffers.os_closed {
                    return (Some(None), false);
                }

                return (None, false);
            });
            match ret {
                Some(maybe_lines) => {
                    for maybe_line in maybe_lines {
                        match maybe_line {
                            Some(line) => {
                                println!("[line ferry] Output line: {}", line);
                                self.os.write_line(line);
                            }
                            None => {
                                self.os.close();
                                self.buffers.write(|buffers| {
                                    buffers.os_closed = true
                                });
                            }
                        }
                    }
                }
                None => {
                    self.p.wait().unwrap();
                    return;
                }
            }
        }
    }
}
