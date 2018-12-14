#![feature(mpsc_select)]

use std::io::BufRead;
use std::io::BufReader;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

fn main() {
    println!("Hello, world!");
    let p = Command::new("yes")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    //let (stdin_tx, stdin_rx) = mpsc::channel();
    let (stdout_tx, stdout_rx) = mpsc::channel();

    thread::spawn(move|| {
        let r = BufReader::new(p.stdout.unwrap());
        for line in r.lines() {
            stdout_tx.send(Some(line)).unwrap();
        }
        stdout_tx.send(None).unwrap();
    });

    std::select! {
        e = stdout_rx.recv() => {
            match(e) {
                Some(line) => {
                    println!("Line: {}", line)
                }
                None => {
                    println!("EOF")
                }
            }
        }
    }
}
