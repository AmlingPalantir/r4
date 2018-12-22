use OperationBe2;
use StreamWrapper;
use opts::OptParserView;
use opts::OptionTrait;
use opts::StringVecOption;
use opts::VarOption;
use std::sync::Arc;
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

impl OperationBe2 for Impl {
    type PreOptions = PreOptions;
    type PostOptions = PostOptions;

    fn options<'a, X: 'static>(opt: &'a mut OptParserView<'a, X, PreOptions>) {
        opt.sub(|p| &mut p.op).match_extra_hard(VarOption::push_string_vec);
    }

    fn wrap_stream(o: &PostOptions, os: Stream) -> Stream {
        let (fe, rbe, wbe) = bgop::new(os);

        let op = o.op.clone();
        thread::spawn(move || {
            let os = Stream::new(wbe);
            let mut os = op.wrap(os);

            loop {
                match rbe.read() {
                    Some(e) => {
                        os.write(e);
                    }
                    None => {
                        os.close();
                        return;
                    }
                }
                if os.rclosed() {
                    rbe.rclose();
                }
            }
        });

        return Stream::new(fe);
    }
}

enum SubOperationOption {
}

impl OptionTrait for SubOperationOption {
    type PreType = Vec<String>;
    type ValType = Arc<StreamWrapper>;

    fn validate(p: Vec<String>) -> Arc<StreamWrapper> {
        let name = p.remove(0);
        let op = super::find(&name);
        return Arc::from(op.validate(p));
    }
}
