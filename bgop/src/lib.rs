extern crate stream;
extern crate wns;

use std::collections::VecDeque;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use stream::StreamTrait;
use wns::WaitNotifyState;

struct OneBuffer {
    buf: VecDeque<Option<Entry>>,
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
    fe_to_be: OneBuffer,
    be_to_fe: OneBuffer,
}

impl BgopState {
    fn new() -> Self {
        return BgopState {
            fe_to_be: OneBuffer::new(),
            be_to_fe: OneBuffer::new(),
        };
    }
}

pub struct BgopRbe {
    state: Arc<WaitNotifyState<BgopState>>,
}

impl BgopRbe {
    pub fn read(&self) -> Option<Entry> {
        return self.state.await(&mut |buffers| {
            if let Some(maybe_e) = buffers.fe_to_be.buf.pop_front() {
                return (Some(maybe_e), true);
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

pub struct BgopWbe {
    state: Arc<WaitNotifyState<BgopState>>,
}

impl BgopWbe {
    fn enqueue(&self, maybe_e: Option<Entry>) {
        self.state.await(&mut |buffers| {
            if buffers.be_to_fe.rclosed {
                return (Some(()), false);
            }
            if buffers.be_to_fe.buf.len() < 1024 {
                buffers.be_to_fe.buf.push_back(maybe_e.clone());
                return (Some(()), true);
            }
            return (None, false);
        });
    }
}

impl StreamTrait for BgopWbe {
    fn write(&mut self, e: Entry) {
        self.enqueue(Some(e));
    }

    fn close(self: Box<BgopWbe>) {
        self.enqueue(None);
    }

    fn rclosed(&mut self) -> bool {
        return self.state.read(|buffers| {
            return buffers.be_to_fe.rclosed;
        });
    }
}

pub struct BgopFe {
    os: Option<Stream>,
    state: Arc<WaitNotifyState<BgopState>>,
}

impl BgopFe {
    fn ferry<R, F: FnMut(bool, &mut BgopState) -> Option<R>>(&mut self, f: &mut F) -> R {
        enum Ret<R> {
            Ferry(Vec<Option<Entry>>),
            Return(R),
        }
        loop {
            let ret = self.state.await(&mut |buffers| {
                if buffers.be_to_fe.buf.len() > 0 {
                    let mut maybe_es = Vec::new();
                    while let Some(maybe_e) = buffers.be_to_fe.buf.pop_front() {
                        maybe_es.push(maybe_e);
                    }
                    return (Some(Ret::Ferry(maybe_es)), true);
                }

                if let Some(ret) = f(self.os.is_none(), buffers) {
                    return (Some(Ret::Return(ret)), true);
                }

                return (None, false);
            });
            match ret {
                Ret::Ferry(maybe_es) => {
                    for maybe_e in maybe_es {
                        match maybe_e {
                            Some(e) => {
                                self.os.as_mut().unwrap().write(e);
                                if self.os.as_mut().unwrap().rclosed() {
                                    self.state.write(|buffers| {
                                        buffers.be_to_fe.rclosed = true;
                                        buffers.be_to_fe.buf.clear();
                                    });
                                    break;
                                }
                            }
                            None => {
                                self.os.take().unwrap().close();
                            }
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
        self.ferry(&mut |_os_closed, buffers| {
            if buffers.fe_to_be.rclosed {
                return Some(());
            }

            if buffers.fe_to_be.buf.len() < 1024 {
                buffers.fe_to_be.buf.push_back(Some(e.clone()));
                return Some(());
            }

            return None;
        });
    }

    fn close(mut self: Box<BgopFe>) {
        self.state.write(|buffers| {
            buffers.fe_to_be.buf.push_back(None);
        });
        self.ferry(&mut |os_closed, _buffers| {
            if os_closed {
                return Some(());
            }
            return None;
        });
    }

    fn rclosed(&mut self) -> bool {
        return self.state.read(|buffers| {
            return buffers.fe_to_be.rclosed;
        });
    }
}

pub fn new(os: Stream) -> (BgopFe, BgopRbe, BgopWbe) {
    let state = Arc::new(WaitNotifyState::new(BgopState::new()));

    let fe = BgopFe {
        os: Some(os),
        state: state.clone(),
    };

    let rbe = BgopRbe {
        state: state.clone(),
    };

    let wbe = BgopWbe {
        state: state.clone(),
    };

    return (fe, rbe, wbe);
}
