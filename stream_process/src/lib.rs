extern crate bgop;
extern crate stream;

use bgop::BgopFe;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use stream::Entry;
use stream::StreamTrait;

pub struct ProcessStream {
    fe: BgopFe,
    p: Child,
}

impl ProcessStream {
    pub fn new<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(args: I) -> Self {
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

        return ProcessStream {
            fe: fe,
            p: p,
        };
    }
}

impl StreamTrait for ProcessStream {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return self.fe.write(e, w);
    }

    fn close(self: Box<ProcessStream>, w: &mut FnMut(Entry) -> bool) {
        let mut s = *self;
        Box::new(s.fe).close(w);
        s.p.wait().unwrap();
    }
}
