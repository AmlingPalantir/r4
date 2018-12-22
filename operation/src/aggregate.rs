use Operation;
use StreamWrapper;
use aggregator::AggregatorState;
use opts::OptionTrait;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["aggregate"];
}

#[derive(Default)]
pub struct Impl {
}

impl Operation for Impl {
    fn validate(&self, args: &mut Vec<String>) -> StreamWrapper {
        parse_opt! {
            args,
            (("a", "agg", "aggregator"), AggregatorsOption, aggs),
        }

        return StreamWrapper::new(move |os| {
            let mut aggs = aggs.clone();

            return os.transform_records(move |r| {
                for (_, state) in aggs.iter_mut() {
                    state.add(r.clone());
                }
                return r;
            }).parse();
        });
    }
}

enum AggregatorsOption {
}

impl OptionTrait for AggregatorsOption {
    type PreType = Vec<String>;
    type ValType = Vec<(String, Box<AggregatorState>)>;

    fn argct() -> usize {
        return 1;
    }

    fn set(p: &mut Vec<String>, a: &[String]) {
        p.push(a[0].clone());
    }

    fn val(p: Vec<String>) -> Vec<(String, Box<AggregatorState>)> {
        return p.iter().map(|a| {
            let (label, a) = match a.find('=') {
                Some(i) => (a[0..i].to_string(), &a[(i + 1)..]),
                None => (a.replace("/", "_"), &a[..]),
            };
            let mut parts = a.split(',');
            let agg = aggregator::find(parts.next().unwrap());
            let args: Vec<&str> = parts.collect();
            let state = agg.state(&args);
            return (label, state);
        }).collect();
    }
}
