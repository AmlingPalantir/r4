use OperationBe2;
use SubOperationOption;
use TwoRecordUnionOption;
use opts::parser::OptParserView;
use opts::vals::OptionalStringOption;
use record::RecordTrait;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    tru: TwoRecordUnionOption,
    lk: OptionalStringOption,
    op: SubOperationOption,
}

impl OperationBe2 for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["with-lines"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        TwoRecordUnionOption::options(&mut opt.sub(|p| &mut p.tru));
        opt.sub(|p| &mut p.lk).match_single(&["lk", "line-key"], OptionalStringOption::set_str);
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        let lk = o.lk.as_ref().map(|s| Arc::from(s as &str)).unwrap_or(Arc::from("LINE"));

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
                            let o = o.clone();
                            let r1 = r.clone();
                            let mut substream = stream::compound(
                                o.op.wr.stream(),
                                stream::transform_records(move |r2| {
                                    return o.tru.union(r1.clone(), r2);
                                }),
                            );
                            // Disregard flow hint as one substream rclosing
                            // does not stop us.
                            substream.write(Entry::Line(r.get_path(&lk).expect_string()), w);
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
