use OperationBe2;
use opts::parser::OptParserView;
use opts::vals::OptionalUsizeOption;
use opts::vals::UnvalidatedRawOption;
use record::Record;
use sorts::SortFe;
use sorts::SortState;
use std::cmp::Ordering;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct SortOptions(UnvalidatedRawOption<Vec<Box<SortState>>>);

impl SortOptions {
    pub fn options<'a>(opt: &mut OptParserView<'a, SortOptions>, aliases: &[&str]) {
        sorts::REGISTRY.single_options(&mut opt.sub(|p| &mut (p.0).0), aliases);
        sorts::REGISTRY.multiple_options(&mut opt.sub(|p| &mut (p.0).0), aliases);
    }
}

impl SortOptionsValidated {
    pub fn cmp(&self) -> Box<Fn(&Record, &Record) -> Ordering> {
        let sorts = self.0.clone();
        return Box::new(move |r1, r2| {
            for sort in sorts.iter() {
                let r = sort.cmp(r1, r2);
                if let Ordering::Equal = r {
                    continue;
                }
                return r;
            }
            return Ordering::Equal;
        });
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    sorts: SortOptions,
    partial: OptionalUsizeOption,
}

impl OperationBe2 for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["sort"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        SortOptions::options(&mut opt.sub(|p| &mut p.sorts), &["k", "key"]);
        opt.match_single(&["l", "lex", "lexical"], |p, a| {
            for a in a.split(',') {
                (p.sorts.0).0.push(sorts::lexical::Impl::init(&[a]));
            }
        });
        opt.match_single(&["n", "num", "numeric"], |p, a| {
            for a in a.split(',') {
                (p.sorts.0).0.push(sorts::numeric::Impl::init(&[a]));
            }
        });
        opt.sub(|p| &mut p.partial).match_single(&["p", "partial"], OptionalUsizeOption::parse);
    }

    fn stream(o: &OptionsValidated) -> Stream {
        struct State {
            cmp: Box<Fn(&Record, &Record) -> Ordering>,
            partial: Option<usize>,
            rs: Vec<Record>,
        }

        return stream::compound(
            stream::parse(),
            stream::closures(
                State {
                    cmp: o.sorts.cmp(),
                    partial: o.partial.clone(),
                    rs: Vec::new(),
                },
                |s, e, _w| {
                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(r) => {
                            s.rs.push(r);
                            if let Some(limit) = s.partial {
                                if s.rs.len() >= 2 * limit {
                                    let cmp = &s.cmp;
                                    s.rs.sort_by(|r1, r2| cmp(r1, r2));
                                    s.rs.truncate(limit);
                                }
                            }
                        }
                        Entry::Line(_line) => {
                            panic!("Unexpected line in SortStream");
                        }
                    }
                    return true;
                },
                |s, w| {
                    let mut s = *s;
                    let cmp = &s.cmp;
                    s.rs.sort_by(|r1, r2| cmp(r1, r2));
                    if let Some(limit) = s.partial {
                        s.rs.truncate(limit);
                    }
                    for r in s.rs {
                        if !w(Entry::Record(r)) {
                            return;
                        }
                    }
                },
            ),
        );
    }
}
