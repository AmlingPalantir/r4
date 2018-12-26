use OperationBe2;
use aggregator::AggregatorState;
use opts::parser::OptParserView;
use record::Record;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Clone)]
#[derive(Default)]
struct AggregatorOptions {
    aggs: Vec<(String, Box<AggregatorState>)>,
}

impl Validates for AggregatorOptions {
    type Target = AggregatorOptions;

    fn validate(self) -> AggregatorOptions {
        return self;
    }
}

impl AggregatorOptions {
    fn options<'a>(opt: &mut OptParserView<'a, AggregatorOptions>) {
        aggregator::REGISTRY.labelled_single_options(&mut opt.sub(|p| &mut p.aggs), &["a", "agg", "aggregator"]);
        aggregator::REGISTRY.labelled_multiple_options(&mut opt.sub(|p| &mut p.aggs), &["a", "agg", "aggregator"]);
    }

    fn aggs(&self) -> Vec<(String, Box<AggregatorState>)> {
        return self.aggs.clone();
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    aggs: AggregatorOptions,
}

impl OperationBe2 for Impl {
    type PreOptions = Options;
    type PostOptions = OptionsValidated;

    fn names() -> Vec<&'static str> {
        return vec!["aggregate"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        AggregatorOptions::options(&mut opt.sub(|p| &mut p.aggs));
    }

    fn stream(o: &OptionsValidated) -> Stream {
        return stream::compound(
            stream::parse(),
            stream::closures(
                o.aggs.aggs(),
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
