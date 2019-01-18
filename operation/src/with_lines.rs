use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::DefaultedStringOption;
use record::RecordTrait;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::SubOperationOption;
use super::TwoRecordUnionOption;

option_defaulters! {
    LineDefaulter: String => "LINE".to_string(),
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    tru: TwoRecordUnionOption,
    lk: DefaultedStringOption<LineDefaulter>,
    op: SubOperationOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.add_sub(|p| &mut p.tru, TwoRecordUnionOption::new_options());
        opt.match_single(&["lk", "line-key"], |p, a| p.lk.set_str(a));
        opt.match_extra_hard(|p, a| p.op.push(a));
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["with-lines"];
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::compound(
            stream::parse(),
            stream::closures(
                (),
                move |_s, e, w| {
                    match e {
                        Entry::Bof(file) => {
                            return w(Entry::Bof(file));
                        }
                        Entry::Record(r) => {
                            let o1 = o.clone();
                            let r1 = r.clone();
                            let mut substream = stream::compound(
                                o.op.wr.stream(),
                                stream::transform_records(move |r2| {
                                    return o1.tru.union(r1.clone(), r2);
                                }),
                            );
                            // Disregard flow hint as one substream rclosing
                            // does not stop us.
                            substream.write(Entry::Line(r.get_path(&o.lk).expect_string()), w);
                            substream.close(w);

                            return true;
                        }
                        Entry::Line(_line) => {
                            panic!("Unexpected line in DeaggregateStream");
                        }
                    }
                },
                |_s, _w| {
                },
            )
        );
    }
}
