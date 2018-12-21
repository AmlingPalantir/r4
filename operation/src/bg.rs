use Operation;
use StreamWrapper;
use bgop::BgopBe;
use bgop::BgopFe;
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use stream::Entry;
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
            let bgop = BgopFe::<Entry>::new(move |maybe_e| {
                match maybe_e {
                    Some(e) => {
                        return os.write(e);
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
                    let os = Stream::new(BgopBeStream(bgop.clone()));
                    let mut os = op.wrap(os);

                    match bgop.read() {
                        Some(e) => {
                            if !os.write(e) {
                                bgop.rclose();
                            }
                        }
                        None => {
                            os.close();
                            return;
                        }
                    }
                });
            }

            return Stream::new(BgopFeStream(bgop));
        });
    }
}

struct BgopBeStream(BgopBe<Entry>);

impl StreamTrait for BgopBeStream {
    fn write(&mut self, e: Entry) -> bool {
        return self.0.write(e);
    }

    fn close(&mut self) {
        self.0.close();
    }
}

struct BgopFeStream(BgopFe<Entry>);

impl StreamTrait for BgopFeStream {
    fn write(&mut self, e: Entry) -> bool {
        return self.0.write(e);
    }

    fn close(&mut self) {
        self.0.close();
    }
}
