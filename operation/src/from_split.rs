use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::DefaultedOption;
use opts::vals::StringVecOption;
use record::Record;
use record::RecordTrait;
use regex::Regex;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;

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

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_single(&["d", "delim"], |p, a| p.delimiter.set(DelimiterOption::String(a.to_string())));
        opt.match_single(&["re", "regex"], |p, a| p.delimiter.set(DelimiterOption::Regex(Arc::new(Regex::new(a)?))));
        opt.match_single(&["k", "keys"], |p, a| p.keys.push_split(a));
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["from-split"];
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::closures(
            (),
            move |_s, e, w| {
                let line = e.deparse();

                let mut r = Record::empty_hash();
                let vals: Vec<_> = match o.delimiter {
                    DelimiterOption::String(ref s) => line.split(s).collect(),
                    DelimiterOption::Regex(ref re) => re.split(&line).collect(),
                };
                for (k, v) in o.keys.iter().zip(vals) {
                    r.set_path(k, Record::from(v));
                }

                return w(Entry::Record(r));
            },
            |_s, _w| {
            },
        );
    }
}
