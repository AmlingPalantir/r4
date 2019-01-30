use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::RequiredStringOption;
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
use validates::Validates;
use validates::ValidationResult;

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

#[derive(Default)]
struct RegexOption(RequiredStringOption);

impl Validates for RegexOption {
    type Target = Arc<Regex>;

    fn validate(self) -> ValidationResult<Arc<Regex>> {
        return Result::Ok(Arc::new(Regex::new(&self.0.validate()?)?));
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    re: RegexOption,
    keys: StringVecOption,
}

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_single(&["re", "regex"], |p, a| p.re.0.set_str(a));
        opt.match_single(&["k", "keys"], |p, a| p.keys.push_split(a));
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["from-regex"];
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::closures(
            (),
            move |_s, e, w| {
                let line = e.deparse();

                if let Some(m) = o.re.captures(&line) {
                    let mut r = Record::empty_hash();

                    let ki = o.keys.iter();
                    let gi = m.iter().skip(1);
                    for (k, g) in ki.zip(gi) {
                        if let Some(m) = g {
                            r.set_path(&k, Record::from(m.as_str()));
                        }
                    }

                    return w(Entry::Record(r));
                }

                return true;
            },
            |_s, _w| {
            },
        );
    }
}
