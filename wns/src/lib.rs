use std::sync::Condvar;
use std::sync::Mutex;

pub struct WaitNotifyState<S> {
    c: Condvar,
    m: Mutex<S>,
}

impl<S> WaitNotifyState<S> {
    pub fn new(s: S) -> WaitNotifyState<S> {
        return WaitNotifyState {
            c: Condvar::new(),
            m: Mutex::new(s),
        };
    }

    pub fn read<F, R>(&self, f: F) -> R where F: FnOnce(&S) -> R {
        let mg = self.m.lock().unwrap();
        return f(&mg);
    }

    pub fn write<F, R>(&self, f: F) -> R where F: FnOnce(&mut S) -> R {
        let mut mg = self.m.lock().unwrap();
        self.c.notify_all();
        return f(&mut mg);
    }

    pub fn await<F, R>(&self, f: &mut F) -> R where F: FnMut(&mut S) -> (Option<R>, bool) {
        let mut mg = self.m.lock().unwrap();
        loop {
            let (r, n) = f(&mut mg);
            if n {
                self.c.notify_all();
            }
            if let Some(r) = r {
                return r;
            }
            mg = self.c.wait(mg).unwrap();
        }
    }
}
