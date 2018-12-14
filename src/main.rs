use std::env;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::io;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc;
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

enum ChannelElement {
    Line(String),
    AllowInput,
    End,
}

struct ProcessStream {
    os: Box<Stream>,
    stdin_space: u8,
    stdin_tx: Sender<Option<String>>,
    stdout_rx: Receiver<ChannelElement>,
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

        let (stdin_tx, stdin_rx) = mpsc::channel::<Option<String>>();
        let (stdout_tx, stdout_rx) = mpsc::channel();

        let p_stdin = p.stdin;
        let stdout_tx_1 = stdout_tx.clone();
        thread::spawn(move|| {
            let mut r = LineWriter::new(p_stdin.unwrap());
            loop {
                let e = stdin_rx.recv().unwrap();
                match e {
                    Some(line) => {
                        let mut bytes = line.into_bytes();
                        bytes.push(b'\n');
                        r.write_all(&bytes).unwrap();
                        stdout_tx_1.send(ChannelElement::AllowInput).unwrap();
                    }
                    None => {
                        return;
                    }
                };
            }
        });

        let p_stdout = p.stdout;
        let stdout_tx_2 = stdout_tx.clone();
        thread::spawn(move|| {
            let r = BufReader::new(p_stdout.unwrap());
            for line in r.lines() {
                stdout_tx_2.send(ChannelElement::Line(line.unwrap())).unwrap();
            }
            stdout_tx_2.send(ChannelElement::End).unwrap();
        });

        return ProcessStream {
            os: os,
            stdin_space: 1,
            stdin_tx: stdin_tx,
            stdout_rx: stdout_rx,
        };
    }
}

impl Stream for ProcessStream {
    fn write_line(&mut self, line: String) {
        while self.stdin_space == 0 {
            let e = self.stdout_rx.recv().unwrap();
            match e {
                ChannelElement::Line(line) => {
                    println!("[line ferry] Output line: {}", line);
                    self.os.write_line(line);
                }
                ChannelElement::AllowInput => {
                    println!("[line ferry] AllowInput");
                    self.stdin_space += 1;
                }
                ChannelElement::End => {
                    println!("[line ferry] EOF");
                }
            };
        }

        self.stdin_space -= 1;
        self.stdin_tx.send(Some(line)).unwrap();
    }

    fn close(&mut self) {
        self.stdin_tx.send(None).unwrap();
        loop {
            let e = self.stdout_rx.recv().unwrap();
            match e {
                ChannelElement::Line(line) => {
                    println!("[eof ferry] Output line: {}", line);
                    self.os.write_line(line);
                }
                ChannelElement::AllowInput => {
                    println!("[eof ferry] AllowInput");
                    self.stdin_space += 1;
                }
                ChannelElement::End => {
                    println!("[eof ferry] EOF");
                    return;
                }
            };
        }
    }
}
