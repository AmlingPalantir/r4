extern crate bgop;
extern crate stream;

use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use stream::Entry;
use stream::Stream;
use stream::StreamTrait;

pub fn new<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(args: I) -> Stream {
    let mut args = args.into_iter();
    let mut p = Command::new(args.next().unwrap())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let (fe, rbe, mut wbe) = bgop::new();
    let p_stdin = p.stdin.take().unwrap();
    let p_stdout = p.stdout.take().unwrap();

    thread::spawn(move || {
        let mut lw = LineWriter::new(p_stdin);
        loop {
            match rbe.read() {
                Some(Entry::Bof(_file)) => {
                    continue;
                }
                Some(Entry::Record(r)) => {
                    if let Err(_) = writeln!(lw, "{}", r.to_string()) {
                        rbe.rclose();
                    }
                }
                Some(Entry::Line(line)) => {
                    if let Err(_) = writeln!(lw, "{}", line) {
                        rbe.rclose();
                    }
                }
                None => {
                    // drops r
                    return;
                }
            }
        }
    });

    thread::spawn(move || {
        let r = BufReader::new(p_stdout);
        for line in r.lines() {
            let line = line.unwrap();
            if !wbe.write(Entry::Line(Arc::from(line))) {
                break;
            }
        }
        Box::new(wbe).close();
        // return drops r
    });

    return Stream::closures(
        (fe, p),
        |s, e, w| {
            return s.0.write(e, w);
        },
        |s, w| {
            let mut s = *s;
            Box::new(s.0).close(w);
            s.1.wait().unwrap();
        },
    );
}
