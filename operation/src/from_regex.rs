use OperationBe2;
use opts::parser::OptParserView;
use opts::vals::RequiredStringOption;
use opts::vals::StringVecOption;
use record::Record;
use regex::Regex;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
struct RegexOption(RequiredStringOption);

impl Validates for RegexOption {
    type Target = Arc<Regex>;

    fn validate(self) -> Arc<Regex> {
        return Arc::new(Regex::new(&self.0.validate()).unwrap());
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    re: RegexOption,
    keys: StringVecOption,
}

impl OperationBe2 for Impl {
    type PreOptions = Options;
    type PostOptions = OptionsValidated;

    fn names() -> Vec<&'static str> {
        return vec!["from-regex"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.re.0).match_single(&["re", "regex"], RequiredStringOption::set);
        opt.sub(|p| &mut p.keys).match_single(&["k", "keys"], StringVecOption::push_split);
    }

    fn stream(o: &OptionsValidated) -> Stream {
        let re = o.re.clone();
        let keys = o.keys.clone();

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
                            if let Some(m) = re.captures(&line) {
                                let mut r = Record::empty_hash();

                                let ki = keys.iter();
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
