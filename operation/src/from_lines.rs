use OperationBe2;
use opts::parser::OptParserView;
use opts::vals::OptionalStringOption;
use record::Record;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;

pub struct Impl();

declare_opts! {
    lk: OptionalStringOption,
    lnk: OptionalStringOption,
    fk: OptionalStringOption,
}

impl OperationBe2 for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["from-lines"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.lk).match_single(&["lk", "line-key"], OptionalStringOption::set);
        opt.sub(|p| &mut p.lnk).match_single(&["lnk", "lineno-key"], OptionalStringOption::set);
        opt.sub(|p| &mut p.fk).match_single(&["fk", "file-key"], OptionalStringOption::set);
    }

    fn stream(o: &PostOptions) -> Stream {
        let lk = o.lk.clone().unwrap_or_else(|| Arc::from("LINE"));
        let lnk = o.lnk.clone().unwrap_or_else(|| Arc::from("LINENO"));
        let fk = o.fk.clone().unwrap_or_else(|| Arc::from("FILE"));

        return stream::compound(
            stream::deparse(),
            stream::closures(
                (None, 1),
                move |s, e, w| {
                    match e {
                        Entry::Bof(file) => {
                            s.0 = Some(file.clone());
                            s.1 = 1;
                            return w(Entry::Bof(file));
                        }
                        Entry::Record(_r) => {
                            panic!("Unexpected record in FromLinesStream");
                        }
                        Entry::Line(line) => {
                            let mut r = Record::empty_hash();

                            r.set_path(&lk, Record::from_str(line));
                            r.set_path(&lnk, Record::from_i64(s.1));
                            r.set_path(&fk, s.0.clone().map(Record::from_str).unwrap_or_else(Record::null));

                            return w(Entry::Record(r));
                        }
                    }
                },
                |_s, _w| {
                },
            ),
        );
    }
}
