use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

enum ChannelElement {
    Line(String),
    AllowInput,
    End,
}

fn main() {
    println!("Hello, world!");
    let p = Command::new("yes")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let (stdin_tx, stdin_rx) = mpsc::channel();
    let (stdout_tx, stdout_rx) = mpsc::channel();

    let mut stdin_space = 1024;

    let p_stdin = p.stdin;
    let stdout_tx_1 = stdout_tx.clone();
    thread::spawn(move|| {
        let mut r = LineWriter::new(p_stdin.unwrap());
        'E: loop {
            let e = stdin_rx.recv().unwrap();
            match e {
                Some(line) => {
                    r.write_all(line).unwrap();
                    r.write_all(b"\n").unwrap();
                    stdout_tx_1.send(ChannelElement::AllowInput).unwrap()
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

    loop {
        let e = stdout_rx.recv().unwrap();
        match e {
            ChannelElement::Line(line) => {
                println!("Line: {}", line)
            }
            ChannelElement::AllowInput => {
                stdin_space += 1;
            }
            ChannelElement::End => {
                println!("EOF")
            }
        };
    }
}
