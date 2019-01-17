use opts::parser::OptParserView;
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

impl OperationBe2 for ImplBe2 {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["from-regex"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.re.0).match_single(&["re", "regex"], RequiredStringOption::set_str);
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
                            panic!("Unexpected record in FromRegexStream");
                        }
                        Entry::Line(line) => {
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
                        }
                    }
                },
                |_s, _w| {
                },
            ),
        );
    }
}
