use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use std::sync::Arc;
use stream::Stream;
use super::ClumperOptions;
use super::OperationBe;
use super::OperationRegistrant;
use super::SubOperationOption;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    cl: ClumperOptions,
    op: SubOperationOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl Optionsable for ImplBe {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_extra_hard(|p, a| p.op.push(a), "operation to run in each bucket");
        opt.add_sub(|p| &mut p.cl, ClumperOptions::new_options());
    }
}

impl OperationBe for ImplBe {
    fn names() -> Vec<&'static str> {
        return vec!["multiplex"];
    }

    fn help_msg() -> &'static str {
        return "bucket records and run an operation within each bucket";
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return o.op.extra.clone();
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        let o2 = o.clone();
        return o.cl.stream(move |bucket| {
            let s = stream::transform_records(move |mut r| {
                for (path, v) in &bucket {
                    r.set_path(&path, v.clone());
                }
                return r;
            });
            return stream::compound(o2.op.wr.stream(), s);
        });
    }
}
