use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::DefaultedStringOption;
use record::RecordTrait;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe;
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

pub(crate) struct ImplBe();

impl Optionsable for ImplBe {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.add_sub(|p| &mut p.tru, TwoRecordUnionOption::new_options());
        opt.match_single(&["lk", "line-key"], |p, a| p.lk.set_str(a), "key to read lines from (default: 'LINE')");
        opt.match_extra_hard(|p, a| p.op.push(a), "operation to run on each line");
    }
}

impl OperationBe for ImplBe {
    fn names() -> Vec<&'static str> {
        return vec!["expand-lines"];
    }

    fn help_msg() -> &'static str {
        return "run an operation on individual lines, themselves read from input record values";
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return o.op.extra.clone();
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::closures(
            (),
            move |_s, e, w| {
                let r1 = e.parse();

                let o1 = o.clone();
                let line = r1.get_path(&o.lk).expect_string();
                let mut substream = stream::compound(
                    o.op.wr.stream(),
                    stream::transform_records(move |r2| {
                        return o1.tru.union(r1.clone(), r2);
                    }),
                );
                substream.write(Entry::Line(line), w);
                substream.close(w);

                return true;
            },
            |_s, _w| {
            },
        );
    }
}
