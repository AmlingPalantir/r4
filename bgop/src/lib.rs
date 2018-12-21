extern crate wns;

use std::collections::VecDeque;
use std::sync::Arc;
use wns::WaitNotifyState;

struct OneBuffer<E> {
    buf: VecDeque<Option<E>>,
    rclosed: bool,
}

impl<E> OneBuffer<E> {
    fn new() -> Self {
        return OneBuffer {
            buf: VecDeque::new(),
            rclosed: false,
        };
    }
}

struct BgopState<E> {
    os_closed: bool,
    fe_to_be: OneBuffer<E>,
    be_to_fe: OneBuffer<E>,
}

impl<E> BgopState<E> {
    fn new() -> Self {
        return BgopState {
            os_closed: false,
            fe_to_be: OneBuffer::new(),
            be_to_fe: OneBuffer::new(),
        };
    }
}

#[derive(Clone)]
pub struct BgopBe<E: Clone> {
    state: Arc<WaitNotifyState<BgopState<E>>>,
}

impl<E: Clone> BgopBe<E> {
    pub fn read(&self) -> Option<E> {
        return self.state.await(&mut |buffers| {
            if let Some(maybe) = buffers.fe_to_be.buf.pop_front() {
                return (Some(maybe), true);
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

    pub fn write(&self, e: E) -> bool {
        return self.state.await(&mut |buffers| {
            if buffers.be_to_fe.rclosed {
                return (Some(false), false);
            }
            if buffers.be_to_fe.buf.len() < 1024 {
                buffers.be_to_fe.buf.push_back(Some(e.clone()));
                return (Some(true), true);
            }
            return (None, false);
        });
    }

    pub fn close(&self) {
        self.state.write(|buffers| {
            buffers.be_to_fe.buf.push_back(None);
        });
    }
}

pub struct BgopFe<E: Clone> {
    os: Box<FnMut(Option<E>) -> bool>,
    state: Arc<WaitNotifyState<BgopState<E>>>,
}

impl<E: Clone> BgopFe<E> {
    pub fn new<OS: FnMut(Option<E>) -> bool + 'static>(os: OS) -> Self {
        return Self::new_box(Box::new(os));
    }

    pub fn new_box(os: Box<FnMut(Option<E>) -> bool>) -> Self {
        return BgopFe {
            os: os,
            state: Arc::new(WaitNotifyState::new(BgopState::new())),
        }
    }

    pub fn be(&self) -> BgopBe<E> {
        return BgopBe {
            state: self.state.clone(),
        };
    }

    fn ferry<R, F: FnMut(&mut BgopState<E>) -> Option<R>>(&mut self, f: &mut F) -> R {
        enum Ret<E, R> {
            Ferry(Vec<Option<E>>),
            Return(R),
        }
        loop {
            let ret = self.state.await(&mut |buffers| {
                if buffers.be_to_fe.buf.len() > 0 {
                    let mut es = Vec::new();
                    while let Some(e) = buffers.be_to_fe.buf.pop_front() {
                        if e.is_none() {
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

    pub fn write(&mut self, e: E) -> bool {
        return self.ferry(&mut |buffers| {
            if buffers.fe_to_be.rclosed {
                return Some(false);
            }

            if buffers.fe_to_be.buf.len() < 1024 {
                buffers.fe_to_be.buf.push_back(Some(e.clone()));
                return Some(true);
            }

            return None;
        });
    }

    pub fn close(&mut self) {
        self.state.write(|buffers| {
            buffers.fe_to_be.buf.push_back(None);
        });
        self.ferry(&mut |buffers| {
            if buffers.os_closed {
                return Some(());
            }
            return None;
        });
    }
}
