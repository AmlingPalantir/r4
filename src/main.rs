use std::process::Command;
use std::process::Stdio;
use std::select;

fn main() {
    println!("Hello, world!");
    let p = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    select! {
    }
}
