use OperationBe2;
use opts::parser::OptParserView;
use opts::vals::DefaultedOption;
use opts::vals::StringVecOption;
use record::Record;
use record::RecordTrait;
use regex::Regex;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Clone)]
enum DelimiterOption {
    String(String),
    Regex(Arc<Regex>),
}

option_defaulters! {
    CommaDefaulter: DelimiterOption => DelimiterOption::String(",".to_string()),
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    delimiter: DefaultedOption<DelimiterOption, CommaDefaulter>,
    keys: StringVecOption,
}

impl OperationBe2 for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["from-split"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.match_single(&["d", "delim"], |p, a| p.delimiter.set(DelimiterOption::String(a.to_string())));
        opt.match_single(&["re", "regex"], |p, a| p.delimiter.set(DelimiterOption::Regex(Arc::new(Regex::new(a).unwrap()))));
        opt.sub(|p| &mut p.keys).match_single(&["k", "keys"], StringVecOption::push_split);
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::compound(
            stream::deparse(),
            stream::closures(
                (),
                move |_s, e, w| {
                    match e {
                        Entry::Bof(file) => {
                            return w(Entry::Bof(file));
                        }
                        Entry::Record(_r) => {
                            panic!("Unexpected record in FromSplitStream");
                        }
                        Entry::Line(line) => {
                            let mut r = Record::empty_hash();
                            let vals: Vec<_> = match o.delimiter {
                                DelimiterOption::String(ref s) => line.split(s).collect(),
                                DelimiterOption::Regex(ref re) => re.split(&line).collect(),
                            };
                            for (k, v) in o.keys.iter().zip(vals) {
                                r.set_path(k, Record::from(v));
                            }
                            return w(Entry::Record(r));
                        }
                    }
                },
                |_s, _w| {
                },
            ),
        );
    }
}
