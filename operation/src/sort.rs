use opts::parser::OptParserView;
use opts::vals::OptionalUsizeOption;
use opts::vals::UnvalidatedOption;
use record::Record;
use registry::Registrant;
use sorts::BoxedSort;
use sorts::bucket::SortBucket;
use sorts::bucket::VecDequeSortBucket;
use std::collections::HashMap;
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

pub struct GenericSortBucket<T> {
    ts: HashMap<usize, T>,
    i: usize,
    bucket: Box<SortBucket>,
}

impl<T> GenericSortBucket<T> {
    pub fn add(&mut self, r: Record, t: T) {
        let i = self.i;
        self.i += 1;

        self.ts.insert(i, t);
        self.bucket.add(r, i);
    }

    fn removed(&mut self, e: Option<(Record, usize)>) -> Option<(Record, T)> {
        return match e {
            Some((r, i)) => Some((r, self.ts.remove(&i).unwrap())),
            None => None,
        };
    }

    pub fn remove_last(&mut self) -> Option<(Record, T)> {
        let e = self.bucket.remove_last();
        return self.removed(e);
    }

    pub fn remove_first(&mut self) -> Option<(Record, T)> {
        let e = self.bucket.remove_first();
        return self.removed(e);
    }

    pub fn size(&self) -> usize {
        return self.ts.len();
    }
}

impl SortOptionsValidated {
    pub fn new_bucket<T>(&self) -> GenericSortBucket<T> {
        let f: Rc<Fn() -> Box<SortBucket>> = Rc::new(VecDequeSortBucket::new);

        let f = self.0.iter().rev().fold(f, |f, sort| {
            let sort = sort.clone();
            return Rc::new(move || sort.new_bucket(f.clone()));
        });

        return GenericSortBucket {
            ts: HashMap::new(),
            i: 0,
            bucket: f(),
        };
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
            rs: GenericSortBucket<()>,
        }

        let rs = o.sorts.new_bucket();

        return stream::compound(
            stream::parse(),
            stream::closures(
                State {
                    o: o,
                    rs: rs,
                },
                |s, e, _w| {
                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(r) => {
                            s.rs.add(r, ());
                            if let Some(limit) = s.o.partial {
                                if s.rs.size() > limit {
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
