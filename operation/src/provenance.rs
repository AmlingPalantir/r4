use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::DefaultedStringOption;
use record::Record;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::SubOperationOption;

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

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_single(&["ok", "origin-key"], |p, a| p.ok.set_str(a));
        opt.match_extra_hard(|p, a| p.op.push(a));
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["provenance"];
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
