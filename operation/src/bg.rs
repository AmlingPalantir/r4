use Operation;
use StreamWrapper;
use bgop::BgopFe;
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use stream::Stream;
use stream::StreamTrait;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["bg"];
}

#[derive(Default)]
pub struct Impl {
}

impl Operation for Impl {
    fn validate(&self, args: &mut VecDeque<String>) -> StreamWrapper {
        let name = args.pop_front().unwrap();
        let op = super::find(&name);
        let op = op.validate(args);
        let op = Arc::from(op);

        return StreamWrapper::new(move |mut os| {
            let bgop = BgopFe::new(move |maybe_e| {
                match maybe_e {
                    Some(e) => {
                        os.write(e);
                        return !os.rclosed();
                    }
                    None => {
                        os.close();
                        return false;
                    }
                }
            });

            {
                let bgop = bgop.be();
                let op = op.clone();
                thread::spawn(move || {
                    let os = Stream::new(bgop.clone());
                    let mut os = op.wrap(os);

                    loop {
                        match bgop.read() {
                            Some(e) => {
                                os.write(e);
                                if os.rclosed() {
                                    bgop.rclose();
                                }
                            }
                            None => {
                                os.close();
                                return;
                            }
                        }
                    }
                });
            }

            return Stream::new(bgop);
        });
    }
}
