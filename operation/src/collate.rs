use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::IntoArcOption;
use std::sync::Arc;
use stream::Stream;
use super::ClumperOptions;
use super::OperationBe;
use super::OperationRegistrant;
use super::aggregate;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    cl: ClumperOptions,
    ag: IntoArcOption<<aggregate::ImplBe as Optionsable>::Options>,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl Optionsable for ImplBe {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.add_sub(|p| &mut p.cl, ClumperOptions::new_options());
        opt.add_sub(|p| &mut p.ag.0, aggregate::ImplBe::new_options());
    }
}

impl OperationBe for ImplBe {
    fn names() -> Vec<&'static str> {
        return vec!["collate"];
    }

    fn help_msg() -> &'static str {
        return "bucket records and compute aggregates within each bucket";
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return aggregate::ImplBe::get_extra(o.ag.clone());
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
            return stream::compound(aggregate::ImplBe::stream(o2.ag.clone()), s);
        });
    }
}
