use OperationBe;
use SubOperationOption;
use opts::parser::OptParserView;
use std::thread;
use stream::Stream;

pub struct Impl();

declare_opts! {
    op: SubOperationOption,
}

impl OperationBe for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn names() -> Vec<&'static str> {
        return vec!["bg"];
    }

    fn options<'a>(mut opt: OptParserView<'a, PreOptions>) {
        opt.sub(|p| &mut p.op).match_extra_hard(SubOperationOption::push);
    }

    fn get_extra(o: &PostOptions) -> &Vec<String> {
        return &o.op.extra;
    }

    fn stream(o: &PostOptions) -> Stream {
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
