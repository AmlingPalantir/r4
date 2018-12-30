use opts::parser::OptParserView;
use opts::vals::RequiredStringOption;
use opts::vals::UnvalidatedOption;
use record::Record;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::TwoRecordUnionOption;
use validates::Validates;

#[derive(Default)]
struct DbOption {
    pairs: UnvalidatedOption<Vec<(String, String)>>,
    file: RequiredStringOption,
}

impl Validates for DbOption {
    type Target = Db;

    fn validate(self) -> Db {
        return Db::new(&self.file.validate(), &self.pairs.validate());
    }
}

#[derive(Clone)]
struct Db {
    db: HashMap<Vec<Record>, (bool, Vec<Record>)>,
    rks: Arc<Vec<String>>,
}

impl Db {
    fn new(file: &str, pairs: &[(String, String)]) -> Db {
        let mut db = Db {
            db: HashMap::new(),
            rks: Arc::new(pairs.iter().map(|(_lk, rk)| rk.clone()).collect()),
        };
        for line in BufReader::new(File::open(&file as &str).unwrap()).lines() {
            let r = Record::parse(&line.unwrap());
            let ks = pairs.iter().map(|(lk, _rk)| r.get_path(lk)).collect();
            db.db.entry(ks).or_insert_with(|| (false, Vec::new())).1.push(r);
        }
        return db;
    }

    fn query(&mut self, r: &Record) -> Option<impl Iterator<Item = &Record>> {
        let ks: Vec<_> = self.rks.iter().map(|rk| r.get_path(rk)).collect();
        return self.db.get_mut(&ks).map(|e| {
            e.0 = true;
            return e.1.iter();
        });
    }

    fn leftover(&self) -> impl Iterator<Item = &Record> {
        return self.db.values().filter(|e| !e.0).flat_map(|e| e.1.iter());
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    tru: TwoRecordUnionOption,
    fills: UnvalidatedOption<(bool, bool)>,
    db: DbOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl OperationBe2 for ImplBe2 {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["join"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        TwoRecordUnionOption::options(&mut opt.sub(|p| &mut p.tru));
        opt.match_zero(&["inner"], |p| p.fills.0 = (false, false));
        opt.match_zero(&["left"], |p| p.fills.0 = (true, false));
        opt.match_zero(&["right"], |p| p.fills.0 = (false, true));
        opt.match_zero(&["outer"], |p| p.fills.0 = (true, true));
        opt.match_n(&["on"], 2, |p, a| p.db.pairs.0.push((a[0].to_string(), a[1].to_string())));
        opt.sub(|p| &mut p.db.file).match_extra_soft(RequiredStringOption::maybe_set_str);
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        let db = o.db.clone();
        let o1 = o;
        let o2 = o1.clone();

        return stream::compound(
            stream::parse(),
            stream::closures(
                db,
                move |s, e, w| {
                    match e {
                        Entry::Bof(_file) => {
                            return true;
                        }
                        Entry::Record(r) => {
                            match s.query(&r) {
                                Some(r2s) => {
                                    for r2 in r2s {
                                        if !w(Entry::Record(o1.tru.union(r2.clone(), r.clone()))) {
                                            return false;
                                        }
                                    }
                                }
                                None => {
                                    if o1.fills.0 {
                                        if !w(Entry::Record(o1.tru.union_maybe(None, Some(r)))) {
                                            return false;
                                        }
                                    }
                                }
                            }
                            return true;
                        }
                        Entry::Line(_line) => {
                            panic!("Unexpected line in JoinStream");
                        }
                    }
                },
                move |s, w| {
                    if o2.fills.1 {
                        for r2 in s.leftover() {
                            if !w(Entry::Record(o2.tru.union_maybe(Some(r2.clone()), None))) {
                                return;
                            }
                        }
                    }
                },
            ),
        );
    }
}
