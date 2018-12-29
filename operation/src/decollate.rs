use OperationBe2;
use deaggregator::DeaggregatorState;
use opts::parser::OptParserView;
use opts::vals::UnvalidatedRawOption;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    deaggs: UnvalidatedRawOption<Vec<Box<DeaggregatorState>>>
}

impl OperationBe2 for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["decollate"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        deaggregator::REGISTRY.single_options(&mut opt.sub(|p| &mut p.deaggs.0), &["d", "deagg", "deaggregator"]);
        deaggregator::REGISTRY.multiple_options(&mut opt.sub(|p| &mut p.deaggs.0), &["d", "deagg", "deaggregator"]);
    }

    fn stream(o: &OptionsValidated) -> Stream {
        return o.deaggs.iter().fold(stream::parse(), |s, deagg| {
            let deagg = deagg.clone();
            return stream::compound(
                s,
                stream::closures(
                    (),
                    move |_s, e, w| {
                        match e {
                            Entry::Bof(file) => {
                                return w(Entry::Bof(file));
                            }
                            Entry::Record(r) => {
                                for pairs in deagg.deaggregate(r.clone()) {
                                    let mut r2 = r.clone();
                                    for (k, v) in pairs {
                                        r2.set_path(&k, v);
                                    }
                                    if !w(Entry::Record(r2)) {
                                        return false;
                                    }
                                }
                                return true;
                            }
                            Entry::Line(_line) => {
                                panic!("Unexpected line in DeaggregateStream");
                            }
                        }
                    },
                    |_s, _w| {
                    },
                )
            );
        });
    }
}
