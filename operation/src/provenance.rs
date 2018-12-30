use opts::parser::OptParserView;
use opts::vals::DefaultedStringOption;
use record::Record;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::SubOperationOption;
use validates::Validates;

option_defaulters! {
    OriginDefaulter: String => "ORIGIN".to_string(),
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    ok: DefaultedStringOption<OriginDefaulter>,
    op: SubOperationOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl OperationBe2 for ImplBe2 {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["provenance"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.ok).match_single(&["ok", "origin-key"], DefaultedStringOption::set_str);
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::closures(
            o.op.wr.stream(),
            move |s, e, w| {
                match e {
                    Entry::Bof(file) => {
                        return s.write(Entry::Bof(file), w);
                    }
                    Entry::Record(r) => {
                        return s.write(Entry::Record(r.clone()), &mut |mut e| {
                            if let Entry::Record(ref mut r2) = e {
                                r2.set_path(&o.ok, r.clone());
                            }
                            return w(e);
                        });
                    }
                    Entry::Line(line) => {
                        return s.write(Entry::Line(line.clone()), &mut |mut e| {
                            if let Entry::Record(ref mut r2) = e {
                                r2.set_path(&o.ok, Record::from(line.clone()));
                            }
                            return w(e);
                        });
                    }
                }
            },
            |s, w| {
                s.close(w);
            },
        );
    }
}
