extern crate wns;

use std::collections::VecDeque;
use std::sync::Arc;
use wns::WaitNotifyState;

struct OneBuffer<E> {
    buf: VecDeque<Option<E>>,
    rclosed: bool,
}

impl<E> OneBuffer<E> {
    fn new() -> OneBuffer<E> {
        return OneBuffer {
            buf: VecDeque::new(),
            rclosed: false,
        };
    }
}

struct TwoBuffers<E> {
    os_closed: bool,
    fe_to_be: OneBuffer<E>,
    be_to_fe: OneBuffer<E>,
}

impl<E> TwoBuffers<E> {
    fn new() -> TwoBuffers<E> {
        return TwoBuffers {
            os_closed: false,
            fe_to_be: OneBuffer::new(),
            be_to_fe: OneBuffer::new(),
        };
    }
}

struct BgopState<E> where E: Clone {
    wns: WaitNotifyState<TwoBuffers<E>>,
}

impl<E> BgopState<E> where E: Clone {
    fn new() -> BgopState<E> {
        return BgopState {
            wns: WaitNotifyState::new(TwoBuffers::new()),
        };
    }
}

pub struct BgopBe<E> where E: Clone {
    state: Arc<BgopState<E>>,
}

impl<E> BgopBe<E> where E: Clone {
    pub fn read_line(&self) -> Option<E> {
        return self.state.wns.await(&mut |buffers| {
            if let Some(maybe) = buffers.fe_to_be.buf.pop_front() {
                return (Some(maybe), true);
            }
            return (None, false);
        });
    }

    pub fn rclose(&self) {
        self.state.wns.write(|buffers| {
            buffers.fe_to_be.rclosed = true;
            buffers.fe_to_be.buf.clear();
        });
    }

    pub fn write_line(&self, e: E) -> bool {
        return self.state.wns.await(&mut |buffers| {
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
        self.state.wns.write(|buffers| {
            buffers.be_to_fe.buf.push_back(None);
        });
    }
}

pub struct BgopFe<E, OS> where E: Clone, OS: FnMut(Option<E>) -> bool {
    os: OS,
    state: Arc<BgopState<E>>,
}

impl<E, OS> BgopFe<E, OS> where E: Clone, OS: FnMut(Option<E>) -> bool {
    pub fn new(os: OS) -> BgopFe<E, OS> {
        return BgopFe {
            os: os,
            state: Arc::new(BgopState::new()),
        }
    }

    pub fn be(&self) -> BgopBe<E> {
        return BgopBe {
            state: self.state.clone(),
        };
    }

    fn ferry<F>(&mut self, f: &mut F) where F: FnMut(&mut TwoBuffers<E>) -> bool {
        loop {
            let ret = self.state.wns.await(&mut |buffers| {
                if buffers.be_to_fe.buf.len() > 0 {
                    let mut es = Vec::new();
                    while let Some(e) = buffers.be_to_fe.buf.pop_front() {
                        if e.is_none() {
                            buffers.os_closed = true;
                        }
                        es.push(e);
                    }
                    return (Some(Some(es)), true);
                }

                if f(buffers) {
                    return (Some(None), true);
                }

                return (None, false);
            });
            match ret {
                Some(es) => {
                    for e in es {
                        if !(self.os)(e) {
                            self.state.wns.write(|buffers| {
                                buffers.be_to_fe.rclosed = true;
                                buffers.be_to_fe.buf.clear();
                            });
                            break;
                        }
                    }
                }
                None => {
                    return;
                }
            }
        }
    }

    pub fn write_line(&mut self, e: E) -> bool {
        self.ferry(&mut |buffers| {
            if buffers.fe_to_be.rclosed {
                return true;
            }

            if buffers.fe_to_be.buf.len() < 1024 {
                buffers.fe_to_be.buf.push_back(Some(e.clone()));
                return true;
            }

            return false;
        });
        return self.state.wns.read(|buffers| {
            return !buffers.fe_to_be.rclosed;
        });
    }

    pub fn close(&mut self) {
        self.state.wns.write(|buffers| {
            buffers.fe_to_be.buf.push_back(None);
        });
        self.ferry(&mut |buffers| {
            return buffers.os_closed;
        });
    }
}
