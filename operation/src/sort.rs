use opts::parser::OptParserView;
use opts::vals::OptionalUsizeOption;
use opts::vals::UnvalidatedOption;
use record::Record;
use registry::Registrant;
use sorts::SortInbox;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use validates::Validates;

#[derive(Default)]
#[derive(Validates)]
pub struct SortOptions(UnvalidatedOption<Vec<Box<SortInbox>>>);

impl SortOptions {
    pub fn options<'a>(opt: &mut OptParserView<'a, SortOptions>, aliases: &[&str]) {
        sorts::REGISTRY.single_options(&mut opt.sub(|p| &mut (p.0).0), aliases);
        sorts::REGISTRY.multiple_options(&mut opt.sub(|p| &mut (p.0).0), aliases);
    }
}

impl SortOptionsValidated {
    pub fn sort(&self, rs: &mut [Record]) {
        for sort in self.0.iter().rev() {
            sort.sort(rs);
        }
    }

    pub fn sort_aux<T>(&self, rs: &mut [(Record, T)]) {
        for sort in self.0.iter().rev() {
            let idxs = sort.sort_aux(rs.len(), Box::new(|i| &rs[i].0));
            sorts::reorder(rs, &idxs);
        }
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    sorts: SortOptions,
    partial: OptionalUsizeOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl OperationBe2 for ImplBe2 {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["sort"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        SortOptions::options(&mut opt.sub(|p| &mut p.sorts), &["s", "sort"]);
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

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        struct State {
            o: Arc<OptionsValidated>,
            rs: Vec<Record>,
        }

        return stream::compound(
            stream::parse(),
            stream::closures(
                State {
                    o: o,
                    rs: Vec::new(),
                },
                |s, e, _w| {
                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(r) => {
                            s.rs.push(r);
                            if let Some(limit) = s.o.partial {
                                if s.rs.len() >= 2 * limit {
                                    s.o.sorts.sort(&mut s.rs);
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
                    s.o.sorts.sort(&mut s.rs);
                    if let Some(limit) = s.o.partial {
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
