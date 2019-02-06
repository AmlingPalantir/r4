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
        opt.match_single(&["ok", "origin-key"], |p, a| p.ok.set_str(a), "key to set to original input (default: 'ORIGIN')");
        opt.match_extra_hard(|p, a| p.op.push(a), "operation to run on each input");
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
                let ro = match e.clone() {
                    Entry::Record(r) => r,
                    Entry::Line(line) => Record::from(line),
                };

                return s.write(e, &mut |mut e| {
                    if let Entry::Record(ref mut r2) = e {
                        r2.set_path(&o.ok, ro.clone());
                    }
                    return w(e);
                });
            },
            |s, w| {
                s.close(w);
            },
        );
    }
}
