use OperationBe2;
use SubOperationOption;
use TwoRecordUnionOption;
use TwoRecordUnionOptions;
use opts::parser::OptParserView;
use opts::vals::OptionalStringOption;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;

pub struct Impl();

declare_opts! {
    tru: TwoRecordUnionOption,
    lk: OptionalStringOption,
    op: SubOperationOption,
}

impl OperationBe2 for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["with-lines"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, PreOptions>) {
        TwoRecordUnionOptions::options(&mut opt.sub(|p| &mut p.tru));
        opt.sub(|p| &mut p.lk).match_single(&["lk", "line-key"], OptionalStringOption::set);
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
    }

    fn stream(o: &PostOptions) -> Stream {
        let lk = o.lk.as_ref().map(|s| Arc::from(s as &str)).unwrap_or(Arc::from("LINE"));
        let sub_wr = o.op.wr.clone();
        let tru = Arc::from(o.tru.clone());

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
                            let tru = tru.clone();
                            let r1 = r.clone();
                            let mut substream = stream::compound(
                                sub_wr.stream(),
                                stream::transform_records(move |r2| {
                                    return tru.union(r1.clone(), r2);
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
