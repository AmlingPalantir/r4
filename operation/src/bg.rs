use OperationBe;
use SubOperationOption;
use opts::parser::OptParserView;
use std::thread;
use stream::Stream;
use validates::Validates;

pub struct Impl();

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    op: SubOperationOption,
}

impl OperationBe for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["bg"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
    }

    fn get_extra(o: &OptionsValidated) -> Vec<String> {
        return o.op.extra.clone();
    }

    fn stream(o: &OptionsValidated) -> Stream {
        let (fe, rbe, mut wbe) = bgop::new();

        let sub_wr = o.op.wr.clone();
        thread::spawn(move || {
            let mut os = sub_wr.stream();

            loop {
                match rbe.read() {
                    Some(e) => {
                        if !os.write(e, &mut |e| wbe.write(e)) {
                            rbe.rclose();
                        }
                    }
                    None => {
                        os.close(&mut |e| wbe.write(e));
                        Box::new(wbe).close();
                        return;
                    }
                }
            }
        });

        return fe;
    }
}
