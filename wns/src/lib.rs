use std::sync::Condvar;
use std::sync::Mutex;

pub struct WaitNotifyState<S> {
    c: Condvar,
    m: Mutex<S>,
}

impl<S> WaitNotifyState<S> {
    pub fn new(s: S) -> Self {
        return WaitNotifyState {
            c: Condvar::new(),
            m: Mutex::new(s),
        };
    }

    pub fn read<F: FnOnce(&S) -> R, R>(&self, f: F) -> R {
        let mg = self.m.lock().unwrap();
        return f(&mg);
    }

    pub fn write<F: FnOnce(&mut S) -> R, R>(&self, f: F) -> R {
        let mut mg = self.m.lock().unwrap();
        self.c.notify_all();
        return f(&mut mg);
    }

    pub fn await<F: FnMut(&mut S) -> (Option<R>, bool), R>(&self, f: &mut F) -> R {
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
