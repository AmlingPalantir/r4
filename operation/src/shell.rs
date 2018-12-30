use opts::parser::OptParserView;
use opts::vals::StringVecOption;
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
use super::OperationBe;
use super::OperationRegistrant;

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl OperationBe for ImplBe {
    type Options = StringVecOption;

    fn names() -> Vec<&'static str> {
        return vec!["shell"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, StringVecOption>) {
        opt.match_extra_hard(StringVecOption::push_all);
    }

    fn get_extra(_o: Arc<Vec<String>>) -> Vec<String> {
        return vec![];
    }

    fn stream(o: Arc<Vec<String>>) -> Stream {
        let mut args = o.iter();
        let mut p = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let (fe, rbe, wbe) = bgop::new();
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
                        if writeln!(lw, "{}", r.deparse()).is_err() {
                            rbe.rclose();
                        }
                    }
                    Some(Entry::Line(line)) => {
                        if writeln!(lw, "{}", line).is_err() {
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

        thread::spawn(|| {
            let r = BufReader::new(p_stdout);
            for line in r.lines() {
                let line = line.unwrap();
                if !wbe.write(Entry::Line(Arc::from(line))) {
                    break;
                }
            }
            wbe.close();
            // return drops r
        });

        return stream::closures(
            (fe, p),
            |s, e, w| {
                return s.0.write(e, w);
            },
            |mut s, w| {
                s.0.close(w);
                s.1.wait().unwrap();
            },
        );
    }
}
