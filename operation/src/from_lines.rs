use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::DefaultedStringOption;
use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;

option_defaulters! {
    LineDefaulter: String => "LINE".to_string(),
    LinenoDefaulter: String => "LINENO".to_string(),
    FileDefaulter: String => "FILE".to_string(),
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    lk: DefaultedStringOption<LineDefaulter>,
    lnk: DefaultedStringOption<LinenoDefaulter>,
    fk: DefaultedStringOption<FileDefaulter>,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_single(&["lk", "line-key"], |p, a| p.lk.set_str(a));
        opt.match_single(&["lnk", "lineno-key"], |p, a| p.lnk.set_str(a));
        opt.match_single(&["fk", "file-key"], |p, a| p.fk.set_str(a));
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["from-lines"];
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
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

                            r.set_path(&o.lk, Record::from(line));
                            r.set_path(&o.lnk, Record::from(s.1));
                            s.1 += 1;
                            r.set_path(&o.fk, s.0.clone().map(Record::from).unwrap_or_else(Record::null));

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
