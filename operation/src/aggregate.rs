use OperationBe2;
use aggregator::AggregatorState;
use opts::OptParserView;
use opts::UnvalidatedOption;
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

declare_opts! {
    aggs: UnvalidatedOption<Vec<(String, Box<AggregatorState>)>>,
}

impl OperationBe2 for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.aggs).match_single(&["a", "agg", "aggregator"], |aggs, a| {
            let (label, a) = match a.find('=') {
                Some(i) => (a[0..i].to_string(), &a[(i + 1)..]),
                None => (a.replace("/", "_"), &a[..]),
            };
            let mut parts = a.split(',');
            let agg = aggregator::find(parts.next().unwrap());
            let args: Vec<&str> = parts.collect();
            let state = agg.state(&args);
            aggs.push((label, state));
        });
    }

    fn stream(o: &PostOptions) -> Stream {
        let s = Stream::new(AggregateStream {
            aggs: o.aggs.clone(),
        });
        return Stream::compound(Stream::parse(), s);
    }
}

struct AggregateStream {
    aggs: Vec<(String, Box<AggregatorState>)>,
}

impl StreamTrait for AggregateStream {
    fn write(&mut self, e: Entry, _w: &mut FnMut(Entry) -> bool) -> bool {
        match e {
            Entry::Bof(_file) => {
            }
            Entry::Record(r) => {
                for (_, ref mut state) in self.aggs.iter_mut() {
                    state.add(r.clone());
                }
            }
            Entry::Line(_line) => {
                panic!("Unexpected line in AggregateStream");
            }
        }
        return true;
    }

    fn close(self: Box<AggregateStream>, w: &mut FnMut(Entry) -> bool) {
        let s = *self;
        let mut r = Record::empty_hash();
        for (label, state) in s.aggs.into_iter() {
            r.set_path(&label, state.finish());
        }
        w(Entry::Record(r));
    }
}
