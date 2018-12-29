use OperationBe2;
use TwoRecordUnionOption;
use opts::parser::OptParserView;
use opts::vals::RequiredStringOption;
use opts::vals::UnvalidatedRawOption;
use record::Record;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::Deref;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
struct DbOption {
    pairs: UnvalidatedRawOption<Vec<(String, String)>>,
    file: RequiredStringOption,
}

impl Validates for DbOption {
    type Target = Arc<Db>;

    fn validate(self) -> Arc<Db> {
        return Arc::new(Db::new(self.file.validate(), self.pairs.validate()));
    }
}

#[derive(Clone)]
struct Db {
    db: HashMap<Vec<Record>, (bool, Vec<Record>)>,
    rks: Arc<Vec<String>>,
}

impl Db {
    fn new(file: Arc<str>, pairs: Vec<(String, String)>) -> Db {
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
        let ks: Vec<Record> = self.rks.iter().map(|rk| r.get_path(rk)).collect();
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
    fills: UnvalidatedRawOption<(bool, bool)>,
    db: DbOption,
}

impl OperationBe2 for Impl {
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
        opt.sub(|p| &mut p.db.file).match_extra_soft(RequiredStringOption::maybe_set);
    }

    fn stream(o: impl Deref<Target = OptionsValidated>) -> Stream {
        let db = (*o.db).clone();
        let tru = o.tru.clone();
        let fill_left = o.fills.0;
        let fill_right = o.fills.1;

        return stream::compound(
            stream::parse(),
            stream::closures(
                (db, tru),
                move |s, e, w| {
                    match e {
                        Entry::Bof(_file) => {
                            return true;
                        }
                        Entry::Record(r) => {
                            match s.0.query(&r) {
                                Some(r2s) => {
                                    for r2 in r2s {
                                        if !w(Entry::Record(s.1.union(r2.clone(), r.clone()))) {
                                            return false;
                                        }
                                    }
                                }
                                None => {
                                    if fill_left {
                                        if !w(Entry::Record(s.1.union_maybe(None, Some(r)))) {
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
                    if fill_right {
                        for r2 in s.0.leftover() {
                            if !w(Entry::Record(s.1.union_maybe(Some(r2.clone()), None))) {
                                return;
                            }
                        }
                    }
                },
            ),
        );
    }
}
