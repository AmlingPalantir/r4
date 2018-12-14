use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::io;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

trait Stream {
    fn write_line(&self, String);
    fn close(&self);
}

enum ChannelElement {
    Line(String),
    AllowInput,
    End,
}

fn main() {
    let mut os = StdoutStream::new();
    let mut os = ProcessStream::new(&mut os, &["cat"]);
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
    fn write_line(&self, line: String) {
        println!("StdoutStream line: {}", line);
    }

    fn close(&self) {
    }
}

struct ProcessStream {
    stdin_space: u8,
}

impl ProcessStream {
    fn new(os: &mut Stream, args: &[&str]) -> ProcessStream
    {
        let p = Command::new(args[0])
            .args(&args[1..])
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
            'E: loop {
                let e = stdin_rx.recv().unwrap();
                match e {
                    Some(line) => {
                        r.write_all(&line.into_bytes()).unwrap();
                        r.write_all(b"\n").unwrap();
                        stdout_tx_1.send(ChannelElement::AllowInput).unwrap();
                    }
                    None => {
                        break 'E;
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
            stdin_space: 1,
        };
    }
}

impl Stream for ProcessStream {
    fn write_line(&self, line: String) {
//        while stdin_space == 0 {
//            let e = stdout_rx.recv().unwrap();
//            match e {
//                ChannelElement::Line(line) => {
//                    println!("[line ferry] Output line: {}", line);
//                }
//                ChannelElement::AllowInput => {
//                    println!("[line ferry] AllowInput");
//                    stdin_space += 1;
//                }
//                ChannelElement::End => {
//                    println!("[line ferry] EOF");
//                }
//            };
//        }
//
//        stdin_space -= 1;
//        stdin_tx.send(Some(line)).unwrap();
    }

    fn close(&self) {
//    stdin_tx.send(None).unwrap();
//    'E: loop {
//        let e = stdout_rx.recv().unwrap();
//        match e {
//            ChannelElement::Line(line) => {
//                println!("[eof ferry] Output line: {}", line);
//            }
//            ChannelElement::AllowInput => {
//                println!("[eof ferry] AllowInput");
//                stdin_space += 1;
//            }
//            ChannelElement::End => {
//                println!("[eof ferry] EOF");
//                break 'E;
//            }
//        };
//    }
    }
}
