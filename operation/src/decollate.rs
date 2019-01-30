use deaggregator::BoxedDeaggregator;
use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::UnvalidatedOption;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    deaggs: UnvalidatedOption<Vec<BoxedDeaggregator>>
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.add_sub(|p| &mut p.deaggs.0, deaggregator::REGISTRY.single_options(&["d", "deagg", "deaggregator"]));
        opt.add_sub(|p| &mut p.deaggs.0, deaggregator::REGISTRY.multiple_options(&["d", "deagg", "deaggregator"]));
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["decollate"];
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return o.deaggs.iter().fold(stream::parse(), |s, deagg| {
            let deagg = deagg.clone();
            return stream::compound(
                s,
                stream::closures(
                    (),
                    move |_s, e, w| {
                        let r = e.parse();

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
                    },
                    |_s, _w| {
                    },
                )
            );
        });
    }
}
