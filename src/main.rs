use std::collections::VecDeque;
use std::env;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::io;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;

trait Stream {
    fn write_line(&mut self, String);
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

    fn close(&mut self) {
    }
}

struct ProcessBuffer {
    lines: VecDeque<String>,
    closed: bool,
    rclosed: bool,
}

impl ProcessBuffer {
    fn new() -> ProcessBuffer {
        return ProcessBuffer {
            lines: VecDeque::new(),
            closed: false,
            rclosed: false,
        };
    }
}

struct ProcessBuffers {
    stdin: ProcessBuffer,
    stdout: ProcessBuffer,
}

impl ProcessBuffers {
    fn new() -> ProcessBuffers {
        return ProcessBuffers {
            stdin: ProcessBuffer::new(),
            stdout: ProcessBuffer::new(),
        };
    }
}

struct ProcessStream {
    os: Box<Stream>,
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
                loop {
                    fn read_line(cond: &Condvar, buffers: &Mutex<ProcessBuffers>) -> Option<String> {
                        let mut buffers = buffers.lock().unwrap();
                        loop {
                            match buffers.stdin.lines.pop_front() {
                                None => {
                                }
                                Some(line) => {
                                    cond.notify_all();
                                    return Some(line);
                                }
                            }
                            if buffers.stdin.closed {
                                return None;
                            }
                            buffers = cond.wait(buffers).unwrap();
                        }
                    }
                    let (ref cond, ref buffers) = *buffers;
                    match read_line(cond, buffers) {
                        Some(line) => {
                            println!("[backend] input line {}", line);
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
                            // drops r
                            return;
                        }
                    }
                }
            });
        }

        //let p_stdout = p.stdout;
        //let stdout_tx_2 = stdout_tx.clone();
        //thread::spawn(move|| {
        //    let r = BufReader::new(p_stdout.unwrap());
        //    for line in r.lines() {
        //        stdout_tx_2.send(ChannelElement::Line(line.unwrap())).unwrap();
        //    }
        //    stdout_tx_2.send(ChannelElement::End).unwrap();
        //});

        return ProcessStream {
            os: os,
            buffers: buffers,
        };
    }
}

impl Stream for ProcessStream {
    fn write_line(&mut self, line: String) {
//        loop {
//            if self.end_input {
//                println!("[frontend] input dropped");
//                return;
//            }
//            if self.stdin_space > 0 {
//                println!("[frontend] input ready");
//                break;
//            }
//
//            let e = self.stdout_rx.recv().unwrap();
//            match e {
//                ChannelElement::Line(line) => {
//                    println!("[line ferry] Output line: {}", line);
//                    self.os.write_line(line);
//                }
//                ChannelElement::EndInput => {
//                    println!("[line ferry] EndInput");
//                    self.end_input = true;
//                }
//                ChannelElement::AllowInput => {
//                    println!("[line ferry] AllowInput");
//                    self.stdin_space += 1;
//                }
//                ChannelElement::End => {
//                    println!("[line ferry] EOF");
//                }
//            }
//        }
//
//        self.stdin_space -= 1;
//        self.stdin_tx.send(Some(line)).unwrap();
    }

    fn close(&mut self) {
//        self.stdin_tx.send(None).unwrap();
//        loop {
//            let e = self.stdout_rx.recv().unwrap();
//            match e {
//                ChannelElement::Line(line) => {
//                    println!("[eof ferry] Output line: {}", line);
//                    self.os.write_line(line);
//                }
//                ChannelElement::AllowInput => {
//                    println!("[eof ferry] AllowInput");
//                    self.stdin_space += 1;
//                }
//                ChannelElement::EndInput => {
//                    println!("[eof ferry] EndInput");
//                    self.end_input = true;
//                }
//                ChannelElement::End => {
//                    println!("[eof ferry] EOF");
//                    return;
//                }
//            }
//        }
    }
}
