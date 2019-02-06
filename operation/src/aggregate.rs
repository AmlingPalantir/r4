use aggregator::BoxedAggregator;
use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::BooleanOption;
use opts::vals::UnvalidatedOption;
use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::TwoRecordUnionOption;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    aggs: UnvalidatedOption<Vec<(String, BoxedAggregator)>>,
    tru: TwoRecordUnionOption,
    incremental: BooleanOption,
    no_bucket: BooleanOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.add_sub(|p| &mut p.aggs.0, aggregator::REGISTRY.labelled_single_options(&["a", "agg", "aggregator"]));
        opt.add_sub(|p| &mut p.aggs.0, aggregator::REGISTRY.labelled_multiple_options(&["a", "agg", "aggregator"]));
        opt.add_sub(|p| &mut p.tru, TwoRecordUnionOption::new_options());
        opt.match_zero(&["incremental"], |p| p.incremental.set(), ());
        opt.match_zero(&["no-incremental"], |p| p.incremental.clear(), ());
        opt.match_zero(&["bucket"], |p| p.no_bucket.clear(), ());
        opt.match_zero(&["no-bucket"], |p| p.no_bucket.set(), ());
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["aggregate"];
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        struct State {
            o: Arc<OptionsValidated>,
            aggs: Vec<(String, BoxedAggregator)>,
            recs: Vec<Record>,
        }
        fn aggregate_record(aggs: Vec<(String, BoxedAggregator)>) -> Record {
            let mut rhs = Record::empty_hash();
            for (label, state) in aggs.clone().into_iter() {
                rhs.set_path(&label, state.finish());
            }
            return rhs;
        }

        return stream::closures(
            State {
                o: o.clone(),
                aggs: o.aggs.clone(),
                recs: Vec::new(),
            },
            |s, e, w| {
                let r = e.parse();

                for (_, ref mut state) in s.aggs.iter_mut() {
                    state.add(r.clone());
                }

                if s.o.incremental {
                    if s.o.no_bucket {
                        return w(Entry::Record(s.o.tru.union(r, aggregate_record(s.aggs.clone()))));
                    }

                    return w(Entry::Record(s.o.tru.union_maybe(None, Some(aggregate_record(s.aggs.clone())))));
                }

                if s.o.no_bucket {
                    s.recs.push(r);
                }

                return true;
            },
            |s, w| {
                if s.o.incremental {
                    return;
                }

                let rhs = aggregate_record(s.aggs);

                if !s.o.no_bucket {
                    w(Entry::Record(s.o.tru.union_maybe(None, Some(rhs))));
                    return;
                }

                for lhs in s.recs {
                    if !w(Entry::Record(s.o.tru.union(lhs, rhs.clone()))) {
                        return;
                    }
                }
            },
        );
    }
}
