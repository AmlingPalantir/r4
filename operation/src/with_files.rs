use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::DefaultedStringOption;
use record::Record;
use record::RecordTrait;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe;
use super::OperationRegistrant;
use super::SubOperationOption;

option_defaulters! {
    FileDefaulter: String => "FILE".to_string(),
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    fk: DefaultedStringOption<FileDefaulter>,
    op: SubOperationOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl Optionsable for ImplBe {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_single(&["fk", "file-key"], |p, a| p.fk.set_str(a));
        opt.match_extra_hard(|p, a| p.op.push(a));
    }
}

impl OperationBe for ImplBe {
    fn names() -> Vec<&'static str> {
        return vec!["with-files"];
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return o.op.extra.clone();
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        struct StreamState {
            o: Arc<OptionsValidated>,
            substream: Option<Stream>,
        };
        impl StreamState {
            fn close(&mut self, w: &mut FnMut(Entry) -> bool) {
                if let Some(substream) = self.substream.take() {
                    substream.close(w);
                }
            }

            fn open(&mut self, file: Option<Arc<str>>) -> &mut Stream {
                let fv = match file {
                    Some(file) => Record::from(file),
                    None => Record::null(),
                };
                let o = self.o.clone();
                return self.substream.get_or_insert_with(|| {
                    let o = o.clone();
                    return stream::compound(
                        o.op.wr.stream(),
                        stream::transform_records(move |mut r| {
                            r.set_path(&o.fk, fv.clone());
                            return r;
                        }),
                    );
                });
            }
        }

        return stream::closures(
            StreamState {
                o: o,
                substream: None,
            },
            |s, e, w| {
                match e {
                    Entry::Bof(ref file) => {
                        s.close(w);
                        s.open(Some(file.clone()));
                        return w(Entry::Bof(file.clone()));
                    },
                    e => {
                        // Disregard flow hint as one operation stopping does
                        // not stop us.
                        s.open(None).write(e, w);
                        return true;
                    }
                }
            },
            |mut s, w| {
                s.close(w);
            },
        );
    }
}
