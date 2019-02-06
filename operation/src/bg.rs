use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use std::sync::Arc;
use std::thread;
use stream::Stream;
use super::OperationBe;
use super::OperationRegistrant;
use super::SubOperationOption;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    op: SubOperationOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl Optionsable for ImplBe {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_extra_hard(|p, a| p.op.push(a), ());
    }
}

impl OperationBe for ImplBe {
    fn names() -> Vec<&'static str> {
        return vec!["bg"];
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return o.op.extra.clone();
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        let (fe, rbe, wbe) = bgop::new();

        thread::spawn(move || {
            let mut os = o.op.wr.stream();

            loop {
                match rbe.read() {
                    Some(e) => {
                        if !os.write(e, &mut |e| wbe.write(e)) {
                            rbe.rclose();
                        }
                    }
                    None => {
                        os.close(&mut |e| wbe.write(e));
                        wbe.close();
                        return;
                    }
                }
            }
        });

        return fe;
    }
}
