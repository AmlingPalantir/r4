use opts::parser::OptParserView;
use opts::vals::OptionalUsizeOption;
use registry::Registrant;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::GenericSortBucket;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::SortOptions;

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
                p.sorts.push(sorts::lexical::Impl::init(&[a]));
            }
            return Result::Ok(());
        });
        opt.match_single(&["n", "num", "numeric"], |p, a| {
            for a in a.split(',') {
                p.sorts.push(sorts::numeric::Impl::init(&[a]));
            }
            return Result::Ok(());
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
