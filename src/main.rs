#![feature(mpsc_select)]

use std::process::Command;
use std::process::Stdio;
use std::select;

fn main() {
    println!("Hello, world!");
    let p = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    let (stdin_tx, stdin_rx) = mpsc::channel()
    let (stdout_tx, stdout_rx) = mpsc::channel()

    thread::spawn(move|| {
        ...
    });

    select! {
    }
}
