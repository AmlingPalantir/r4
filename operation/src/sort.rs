use opts::parser::OptParserView;
use opts::vals::OptionalUsizeOption;
use opts::vals::UnvalidatedOption;
use registry::Registrant;
use sorts::BoxedSort;
use sorts::bucket::SortBucket;
use sorts::bucket::VecDequeSortBucket;
use std::rc::Rc;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use validates::Validates;

#[derive(Default)]
#[derive(Validates)]
pub struct SortOptions(UnvalidatedOption<Vec<BoxedSort>>);

impl SortOptions {
    pub fn options<'a>(opt: &mut OptParserView<'a, SortOptions>, aliases: &[&str]) {
        sorts::REGISTRY.single_options(&mut opt.sub(|p| &mut (p.0).0), aliases);
        sorts::REGISTRY.multiple_options(&mut opt.sub(|p| &mut (p.0).0), aliases);
    }
}

impl SortOptionsValidated {
    pub fn new_bucket(&self) -> Box<SortBucket> {
        let f: Rc<Fn() -> Box<SortBucket>> = Rc::new(VecDequeSortBucket::new);

        let f = self.0.iter().rev().fold(f, |f, sort| {
            let sort = sort.clone();
            return Rc::new(move || sort.new_bucket(f.clone()));
        });

        return f();
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
            ri: usize,
            rs: Box<SortBucket>,
        }

        let rs = o.sorts.new_bucket();

        return stream::compound(
            stream::parse(),
            stream::closures(
                State {
                    o: o,
                    ri: 0,
                    rs: rs,
                },
                |s, e, _w| {
                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(r) => {
                            s.rs.add(r, s.ri);
                            s.ri += 1;
                            if let Some(limit) = s.o.partial {
                                if s.ri > limit {
                                    s.rs.remove_last();
                                }
                            }
                        }
                        Entry::Line(_line) => {
                            panic!("Unexpected line in SortStream");
                        }
                    }
                    return true;
                },
                |mut s, w| {
                    while let Some((r, _)) = s.rs.remove_first() {
                        if !w(Entry::Record(r)) {
                            return;
                        }
                    }
                },
            ),
        );
    }
}
