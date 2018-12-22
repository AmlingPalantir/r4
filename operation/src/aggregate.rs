use Operation;
use StreamWrapper;
use aggregator::AggregatorState;
use opts::OptionTrait;
use record::Record;
use stream::Entry;
use stream::Stream;
use stream::StreamTrait;

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
            return Stream::new(AggregateStream {
                os: os,
                aggs: aggs.clone(),
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

struct AggregateStream {
    os: Stream,
    aggs: Vec<(String, Box<AggregatorState>)>,
}

impl StreamTrait for AggregateStream {
    fn write(&mut self, e: Entry) {
        match e {
            Entry::Bof(_file) => {
            }
            Entry::Record(r) => {
                for (_, ref mut state) in self.aggs.iter_mut() {
                    state.add(r.clone());
                }
            }
            Entry::Line(_line) => {
                panic!();
            }
        }
    }

    fn close(self: Box<AggregateStream>) {
        let mut s = *self;
        let mut r = Record::empty_hash();
        for (label, state) in s.aggs.into_iter() {
            r.set_path(&label, state.finish());
        }
        s.os.write(Entry::Record(r));
        s.os.close();
    }

    fn rclosed(&mut self) -> bool {
        return self.os.rclosed();
    }
}
