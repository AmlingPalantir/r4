use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

#[derive(Clone)]
pub struct WaitNotifyState<S> where S: Clone {
    arc: Arc<(Condvar, Mutex<S>)>
}

impl<S> WaitNotifyState<S> where S: Clone {
    pub fn new(s: S) -> WaitNotifyState<S> {
        return WaitNotifyState {
            arc: Arc::new((Condvar::new(), Mutex::new(s))),
        };
    }

    pub fn read<F, R>(&self, f: F) -> R where F: Fn(&S) -> R {
        let (_, ref m) = *self.arc;
        let mg = m.lock().unwrap();
        return f(&mg);
    }

    pub fn write<F, R>(&self, f: F) -> R where F: Fn(&mut S) -> R {
        let (ref c, ref m) = *self.arc;
        let mut mg = m.lock().unwrap();
        c.notify_all();
        return f(&mut mg);
    }

    pub fn await<F, R>(&self, f: F) -> R where F: Fn(&mut S) -> (Option<R>, bool) {
        let (ref c, ref m) = *self.arc;
        let mut mg = m.lock().unwrap();
        loop {
            let (r, n) = f(&mut mg);
            if n {
                c.notify_all();
            }
            if let Some(r) = r {
                return r;
            }
            mg = c.wait(mg).unwrap();
        }
    }
}
