use opts::parser::OptParserView;
use opts::vals::DefaultedStringOption;
use record::RecordTrait;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::SubOperationOption;
use super::TwoRecordUnionOption;
use validates::Validates;

pub struct Impl();

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

impl OperationBe2 for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["with-lines"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        TwoRecordUnionOption::options(&mut opt.sub(|p| &mut p.tru));
        opt.sub(|p| &mut p.lk).match_single(&["lk", "line-key"], DefaultedStringOption::set_str);
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
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
