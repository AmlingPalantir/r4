use opts::parser::OptParserView;
use opts::vals::StringVecOption;
use std::collections::HashSet;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    keys: StringVecOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl OperationBe2 for ImplBe2 {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["to-table"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.keys).match_single(&["k", "key"], StringVecOption::push_split);
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::compound(
            stream::parse(),
            stream::closures(
                Vec::new(),
                |s, e, _w| {
                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(r) => {
                            s.push(r);
                        }
                        Entry::Line(_line) => {
                            panic!("Unexpected line in ToTableStream");
                        }
                    }
                    return true;
                },
                move |s, w| {
                    let mut keys = o.keys.clone();

                    if keys.is_empty() {
                        let mut acc = HashSet::new();

                        for r in s.iter() {
                            for k in r.expect_hash().keys() {
                                acc.insert(k.to_string());
                            }
                        }

                        keys = acc.into_iter().collect();
                    }

                    let mut rows = Vec::new();
                    {
                        let mut row0 = Vec::new();
                        let mut row1 = Vec::new();
                        for (n, key) in keys.iter().enumerate() {
                            if n > 0 {
                                row0.push(("   ".to_string(), ' '));
                                row1.push(("   ".to_string(), ' '));
                            }
                            row0.push((key.to_string(), ' '));
                            row1.push(("".to_string(), '-'));
                        }
                        rows.push(row0);
                        rows.push(row1);
                    }
                    for r in s.iter() {
                        let mut row = Vec::new();
                        for (n, key) in keys.iter().enumerate() {
                            if n > 0 {
                                row.push(("   ".to_string(), ' '));
                            }
                            row.push((r.get_path(key).pretty_string(), ' '));
                        }
                        rows.push(row);
                    }

                    dump_table(&rows, w);
                },
            ),
        );
    }
}

pub fn dump_table(rows: &Vec<Vec<(String, char)>>, w: &mut FnMut(Entry) -> bool) -> bool {
    let mut widths = Vec::new();

    for row in rows {
        for (n, (s, _pad)) in row.iter().enumerate() {
            while n >= widths.len() {
                widths.push(0);
            }
            widths[n] = std::cmp::max(widths[n], s.len());
        }
    }

    for row in rows {
        let mut line = "".to_string();
        for (n, (s, pad)) in row.iter().enumerate() {
            line.push_str(s);
            line.push_str(&str::repeat(&pad.to_string(), widths[n] - s.len()));
        }
        if !w(Entry::Line(Arc::from(line))) {
            return false;
        }
    }

    return true;
}
