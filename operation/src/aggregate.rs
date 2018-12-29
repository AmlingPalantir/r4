use OperationBe2;
use aggregator::AggregatorState;
use opts::parser::OptParserView;
use opts::vals::UnvalidatedRawOption;
use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    aggs: UnvalidatedRawOption<Vec<(String, Box<AggregatorState>)>>,
}

impl OperationBe2 for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["aggregate"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        aggregator::REGISTRY.labelled_single_options(&mut opt.sub(|p| &mut p.aggs.0), &["a", "agg", "aggregator"]);
        aggregator::REGISTRY.labelled_multiple_options(&mut opt.sub(|p| &mut p.aggs.0), &["a", "agg", "aggregator"]);
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::compound(
            stream::parse(),
            stream::closures(
                o.aggs.clone(),
                |s, e, _w| {
                    match e {
                        Entry::Bof(_file) => {
                        }
                        Entry::Record(r) => {
                            for (_, ref mut state) in s.iter_mut() {
                                state.add(r.clone());
                            }
                        }
                        Entry::Line(_line) => {
                            panic!("Unexpected line in AggregateStream");
                        }
                    }
                    return true;
                },
                |s, w| {
                    let mut r = Record::empty_hash();
                    for (label, state) in s.into_iter() {
                        r.set_path(&label, state.finish());
                    }
                    w(Entry::Record(r));
                },
            ),
        );
    }
}
