use OperationBe;
use SubOperationOption;
use opts::OptParserView;
use opts::VarOption;
use std::thread;
use stream::Stream;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["bg"];
}

#[derive(Default)]
pub struct Impl {
}

declare_opts! {
    op: SubOperationOption,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.op).match_extra_hard(VarOption::push_string_vec);
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return &o.op.0;
    }

    fn stream(o: &PostOptions) -> Stream {
        let (fe, rbe, mut wbe) = bgop::new();

        let op = o.op.clone();
        thread::spawn(move || {
            let mut os = op.1.stream();

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

        return Stream::new(fe);
    }
}
