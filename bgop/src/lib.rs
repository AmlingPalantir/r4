extern crate stream;
extern crate wns;

use std::collections::VecDeque;
use std::sync::Arc;
use stream::Entry;
use stream::StreamTrait;
use wns::WaitNotifyState;

struct OneBuffer {
    buf: VecDeque<Entry>,
    rclosed: bool,
}

impl OneBuffer {
    fn new() -> Self {
        return OneBuffer {
            buf: VecDeque::new(),
            rclosed: false,
        };
    }
}

struct BgopState {
    os_closed: bool,
    fe_to_be: OneBuffer,
    be_to_fe: OneBuffer,
}

impl BgopState {
    fn new() -> Self {
        return BgopState {
            os_closed: false,
            fe_to_be: OneBuffer::new(),
            be_to_fe: OneBuffer::new(),
        };
    }
}

#[derive(Clone)]
pub struct BgopBe {
    state: Arc<WaitNotifyState<BgopState>>,
}

impl BgopBe {
    pub fn read(&self) -> Entry {
        return self.state.await(&mut |buffers| {
            if let Some(e) = buffers.fe_to_be.buf.pop_front() {
                return (Some(e), true);
            }
            return (None, false);
        });
    }

    pub fn rclose(&self) {
        self.state.write(|buffers| {
            buffers.fe_to_be.rclosed = true;
            buffers.fe_to_be.buf.clear();
        });
    }
}

impl StreamTrait for BgopBe {
    fn write(&mut self, e: Entry) {
        return self.state.await(&mut |buffers| {
            if buffers.be_to_fe.rclosed {
                return (Some(()), false);
            }
            if buffers.be_to_fe.buf.len() < 1024 {
                buffers.be_to_fe.buf.push_back(e.clone());
                return (Some(()), true);
            }
            return (None, false);
        });
    }

    fn rclosed(&mut self) -> bool {
        return self.state.read(|buffers| {
            return buffers.be_to_fe.rclosed;
        });
    }
}

pub struct BgopFe {
    os: Box<FnMut(Entry) -> bool>,
    state: Arc<WaitNotifyState<BgopState>>,
}

impl BgopFe {
    pub fn new<OS: FnMut(Entry) -> bool + 'static>(os: OS) -> Self {
        return Self::new_box(Box::new(os));
    }

    pub fn new_box(os: Box<FnMut(Entry) -> bool>) -> Self {
        return BgopFe {
            os: os,
            state: Arc::new(WaitNotifyState::new(BgopState::new())),
        }
    }

    pub fn be(&self) -> BgopBe {
        return BgopBe {
            state: self.state.clone(),
        };
    }

    fn ferry<R, F: FnMut(&mut BgopState) -> Option<R>>(&mut self, f: &mut F) -> R {
        enum Ret<R> {
            Ferry(Vec<Entry>),
            Return(R),
        }
        loop {
            let ret = self.state.await(&mut |buffers| {
                if buffers.be_to_fe.buf.len() > 0 {
                    let mut es = Vec::new();
                    while let Some(e) = buffers.be_to_fe.buf.pop_front() {
                        if let Entry::Close() = e {
                            buffers.os_closed = true;
                        }
                        es.push(e);
                    }
                    return (Some(Ret::Ferry(es)), true);
                }

                if let Some(ret) = f(buffers) {
                    return (Some(Ret::Return(ret)), true);
                }

                return (None, false);
            });
            match ret {
                Ret::Ferry(es) => {
                    for e in es {
                        if !(self.os)(e) {
                            self.state.write(|buffers| {
                                buffers.be_to_fe.rclosed = true;
                                buffers.be_to_fe.buf.clear();
                            });
                            break;
                        }
                    }
                }
                Ret::Return(ret) => {
                    return ret;
                }
            }
        }
    }
}

impl StreamTrait for BgopFe {
    fn write(&mut self, e: Entry) {
        self.ferry(&mut |buffers| {
            if buffers.fe_to_be.rclosed {
                return Some(());
            }

            if buffers.fe_to_be.buf.len() < 1024 {
                buffers.fe_to_be.buf.push_back(e.clone());
                return Some(());
            }

            return None;
        });
        if let Entry::Close() = e {
            self.ferry(&mut |buffers| {
                if buffers.os_closed {
                    return Some(());
                }
                return None;
            });
        }
    }

    fn rclosed(&mut self) -> bool {
        return self.state.read(|buffers| {
            return buffers.fe_to_be.rclosed;
        });
    }
}
