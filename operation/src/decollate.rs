use OperationBe2;
use deaggregator::DeaggregatorState;
use opts::parser::OptParserView;
use opts::vals::Validates;
use stream::Entry;
use stream::Stream;

pub struct Impl();

#[derive(Clone)]
#[derive(Default)]
struct DeaggregatorOptions {
    deaggs: Vec<Box<DeaggregatorState>>,
}

impl Validates for DeaggregatorOptions {
    type Target = DeaggregatorOptions;

    fn validate(self) -> DeaggregatorOptions {
        return self;
    }
}

impl DeaggregatorOptions {
    fn options<'a>(opt: &mut OptParserView<'a, DeaggregatorOptions>) {
        deaggregator::REGISTRY.single_options(&mut opt.sub(|p| &mut p.deaggs), &["d", "deagg", "deaggregator"]);
        deaggregator::REGISTRY.multiple_options(&mut opt.sub(|p| &mut p.deaggs), &["d", "deagg", "deaggregator"]);
    }

    fn deaggs(&self) -> Vec<Box<DeaggregatorState>> {
        return self.deaggs.clone();
    }
}

declare_opts! {
    deaggs: DeaggregatorOptions,
}

impl OperationBe2 for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["decollate"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, PreOptions>) {
        DeaggregatorOptions::options(&mut opt.sub(|p| &mut p.deaggs));
    }

    fn stream(o: &PostOptions) -> Stream {
        return o.deaggs.deaggs().iter().fold(stream::parse(), |s, deagg| {
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
