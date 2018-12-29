use OperationBe2;
use opts::parser::OptParserView;
use opts::vals::BooleanOption;
use opts::vals::StringVecOption;
use opts::vals::UnvalidatedArcOption;
use record::Record;
use record::RecordTrait;
use regex::Regex;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    res: UnvalidatedArcOption<Vec<(bool, bool, Vec<String>, Regex)>>,

    keep: StringVecOption,
    keep_all: BooleanOption,

    clobber: BooleanOption,
}

impl OperationBe2 for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["from-multire"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        fn _add_re(p: &mut Options, pre_flush: bool, post_flush: bool, s: &str) {
            match s.find('=') {
                Some(idx) => {
                    let keys = (&s[0..idx]).split(',').map(|s| s.to_string()).collect();
                    let re = Regex::new(&s[(idx + 1)..]).unwrap();
                    p.res.0.push((pre_flush, post_flush, keys, re));
                }
                None => {
                    panic!("No equals in regex spec");
                }
            }
        }

        opt.match_single(&["re"], |p, a| _add_re(p, false, false, a));
        opt.match_single(&["pre"], |p, a| _add_re(p, true, false, a));
        opt.match_single(&["post"], |p, a| _add_re(p, false, true, a));

        opt.sub(|p| &mut p.keep).match_single(&["keep"], StringVecOption::push);
        opt.sub(|p| &mut p.keep_all).match_zero(&["keep-all"], BooleanOption::set);

        opt.sub(|p| &mut p.clobber).match_zero(&["clobber"], BooleanOption::set);
    }

    fn stream(o: &OptionsValidated) -> Stream {
        let res = o.res.clone();
        let clobber1 = o.clobber;
        let clobber2 = o.clobber;

        struct State {
            r: Record,
            keep: Arc<Vec<String>>,
            keep_all: bool,
        }

        impl State {
            fn flush(&mut self, w: &mut FnMut(Entry) -> bool) {
                if !self.r.expect_hash().is_empty() {
                    // We ignore the flow hint, but that's okay as the
                    // surrounding Stream will remember it and at worst we do
                    // the rest of our process for the line.
                    w(Entry::Record(self.r.clone()));
                }

                if self.keep_all {
                    return;
                }

                let mut r2 = Record::empty_hash();
                for path in self.keep.iter() {
                    if self.r.has_path(path) {
                        r2.set_path(path, self.r.get_path(path));
                    }
                }

                self.r = r2;
            }
        }

        return stream::compound(
            stream::deparse(),
            stream::closures(
                State {
                    r: Record::empty_hash(),
                    keep: o.keep.clone(),
                    keep_all: o.keep_all,
                },
                move |s, e, w| {
                    match e {
                        Entry::Bof(file) => {
                            if !clobber1 {
                                s.flush(w);
                            }

                            s.r = Record::empty_hash();

                            return w(Entry::Bof(file));
                        }
                        Entry::Record(_r) => {
                            panic!("Unexpected record in FromMultiRegexStream");
                        }
                        Entry::Line(line) => {
                            for (pre_flush, post_flush, keys, re) in res.iter() {
                                let mut pre_flush = *pre_flush;
                                if let Some(m) = re.captures(&line) {
                                    if !clobber1 {
                                        let ki = keys.iter();
                                        let gi = m.iter().skip(1);
                                        for (k, g) in ki.zip(gi) {
                                            if let Some(_) = g {
                                                if s.r.has_path(&k) {
                                                    pre_flush = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if pre_flush {
                                        s.flush(w);
                                    }
                                    let ki = keys.iter();
                                    let gi = m.iter().skip(1);
                                    for (k, g) in ki.zip(gi) {
                                        if let Some(m) = g {
                                            s.r.set_path(&k, Record::from(m.as_str()));
                                        }
                                    }
                                    if *post_flush {
                                        s.flush(w);
                                    }
                                }
                            }
                            return true;
                        }
                    }
                },
                move |mut s, w| {
                    if !clobber2 {
                        (*s).flush(w);
                    }
                },
            ),
        );
    }
}
