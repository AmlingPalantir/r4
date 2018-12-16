extern crate wns;

use std::collections::VecDeque;
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

pub struct BackgroundOp<E> where E: Clone {
    wns: WaitNotifyState<TwoBuffers<E>>,
}

impl<E> BackgroundOp<E> where E: Clone {
    pub fn new() -> BackgroundOp<E> {
        return BackgroundOp {
            wns: WaitNotifyState::new(TwoBuffers::new()),
        }
    }

    pub fn be_read_line(&self) -> Option<E> {
        return self.wns.await(&mut |buffers| {
            if let Some(maybe) = buffers.fe_to_be.buf.pop_front() {
                return (Some(maybe), true);
            }
            return (None, false);
        });
    }

    pub fn be_rclose(&self) {
        self.wns.write(|buffers| {
            buffers.fe_to_be.rclosed = true;
            buffers.fe_to_be.buf.clear();
        });
    }

    pub fn be_write_line(&self, e: E) -> bool {
        return self.wns.await(&mut |buffers| {
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

    pub fn be_close(&self) {
        self.wns.write(|buffers| {
            buffers.be_to_fe.buf.push_back(None);
        });
    }

    fn fe_ferry<F, F2>(&self, f: &mut F, f2: &mut F2) where F: FnMut(Option<E>), F2: FnMut(&mut TwoBuffers<E>) -> bool {
        loop {
            let ret = self.wns.await(&mut |buffers| {
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

                if f2(buffers) {
                    return (Some(None), true);
                }

                return (None, false);
            });
            match ret {
                Some(es) => {
                    for e in es {
                        f(e);
                    }
                }
                None => {
                    return;
                }
            }
        }
    }

    pub fn fe_write_line<F>(&self, e: E, f: &mut F) where F: FnMut(Option<E>) {
        self.fe_ferry(f, &mut |buffers| {
            if buffers.fe_to_be.rclosed {
                return true;
            }

            if buffers.fe_to_be.buf.len() < 1024 {
                buffers.fe_to_be.buf.push_back(Some(e.clone()));
                return true;
            }

            return false;
        });
    }

    pub fn fe_rclose(&self) {
        self.wns.write(|buffers| {
            buffers.be_to_fe.rclosed = true;
            buffers.be_to_fe.buf.clear();
        });
    }

    pub fn fe_rclosed(&self) -> bool {
        return self.wns.read(|buffers| {
            return buffers.fe_to_be.rclosed;
        });
    }

    pub fn fe_close<F>(&self, f: &mut F) where F: FnMut(Option<E>) {
        self.wns.write(|buffers| {
            buffers.fe_to_be.buf.push_back(None);
        });
        self.fe_ferry(f, &mut |buffers| {
            return buffers.os_closed;
        });
    }
}
