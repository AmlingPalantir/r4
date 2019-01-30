use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::DefaultedStringOption;
use record::RecordTrait;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::SubOperationOption;
use super::TwoRecordUnionOption;

option_defaulters! {
    FileDefaulter: String => "FILE".to_string(),
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    tru: TwoRecordUnionOption,
    fk: DefaultedStringOption<FileDefaulter>,
    op: SubOperationOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.add_sub(|p| &mut p.tru, TwoRecordUnionOption::new_options());
        opt.match_single(&["fk", "file-key"], |p, a| p.fk.set_str(a));
        opt.match_extra_hard(|p, a| p.op.push(a));
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["expand-files"];
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::closures(
            (),
            move |_s, e, w| {
                let r1 = e.parse();

                let o1 = o.clone();
                let file = r1.get_path(&o.fk).coerce_string();
                let mut substream = stream::compound(
                    o.op.wr.stream(),
                    stream::transform_records(move |r2| {
                        return o1.tru.union(r1.clone(), r2);
                    }),
                );
                for line in BufReader::new(File::open(&file as &str).unwrap()).lines() {
                    let line = line.unwrap();
                    if !substream.write(Entry::Line(Arc::from(line)), w) {
                        // flow hint ends substream, but nothing more
                        break;
                    }
                }
                substream.close(w);

                return true;
            },
            |_s, _w| {
            },
        );
    }
}
