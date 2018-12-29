use misc::Either;
use std::sync::Arc;
use std::sync::Mutex;
use super::JsonPrimitive;
use super::Path;
use super::PathStep;
use super::Record;
use super::RecordNode;
use super::RecordTrait;

#[derive(Clone)]
pub struct MRecord(Arc<Mutex<Either<Record, RecordNode<MRecord>>>>);

impl RecordTrait for MRecord {
    fn new(n: RecordNode<Self>) -> Self {
        return MRecord(Arc::new(Mutex::new(Either::Right(n))));
    }

    fn maybe_primitive(&self) -> Option<JsonPrimitive> {
        let n = self.0.lock().unwrap();
        return match *n {
            Either::Left(ref r) => r.maybe_primitive(),
            Either::Right(ref n) => n.maybe_primitive(),
        };
    }
}

impl<T> From<T> for MRecord where RecordNode<MRecord>: From<T> {
    fn from(t: T) -> Self {
        return MRecord::new(RecordNode::from(t));
    }
}

impl MRecord {
    pub fn wrap(r: Record) -> Self {
        return MRecord(Arc::new(Mutex::new(Either::Left(r))));
    }

    pub fn to_record(self) -> Record {
        let n = self.0.lock().unwrap();
        return match *n {
            Either::Left(ref r) => r.clone(),
            Either::Right(ref n) => Record::new(n.clone().map(MRecord::to_record)),
        };
    }

    fn _get_path<'a>(&mut self, mut path: impl Iterator<Item = &'a PathStep<'a>>) -> MRecord {
        match path.next() {
            Some(step) => {
                let mut n = self.0.lock().unwrap();
                let n = (*n).convert_r_mut(|r| {
                    return (*r.0).clone().map(MRecord::wrap);
                });
                return match n.get_rstep_mut(step) {
                    Some(r) => r._get_path(path),
                    None => return MRecord::null(),
                };
            }
            None => {
                return self.clone();
            }
        }
    }

    pub fn get_path(&mut self, path: &str) -> MRecord {
        return self.get_path_obj(&Path::new(path));
    }

    pub fn get_path_obj<'a>(&mut self, path: &Path<'a>) -> MRecord {
        return self._get_path(path.0.iter());
    }

    fn _get_path_fill<'a>(&mut self, mut path: impl Iterator<Item = &'a PathStep<'a>>) -> MRecord {
        match path.next() {
            Some(step) => {
                let mut n = self.0.lock().unwrap();
                let n = (*n).convert_r_mut(|r| {
                    return (*r.0).clone().map(MRecord::wrap);
                });
                return n.get_rstep_fill(step)._get_path_fill(path);
            }
            None => {
                return self.clone();
            }
        }
    }

    pub fn get_path_fill(&mut self, path: &str) -> MRecord {
        return self.get_path_obj_fill(&Path::new(path));
    }

    pub fn get_path_obj_fill<'a>(&mut self, path: &Path<'a>) -> MRecord {
        return self._get_path_fill(path.0.iter());
    }

    fn _set_path<'a>(&mut self, mut path: impl Iterator<Item = &'a PathStep<'a>>, v: MRecord) {
        match path.next() {
            Some(step) => {
                let mut n = self.0.lock().unwrap();
                let n = (*n).convert_r_mut(|r| {
                    return (*r.0).clone().map(MRecord::wrap);
                });
                n.get_rstep_fill(step)._set_path(path, v);
            }
            None => {
                *self = v;
            }
        }
    }

    pub fn set_path(&mut self, path: &str, v: MRecord) {
        self.set_path_obj(&Path::new(path), v);
    }

    pub fn set_path_obj<'a>(&mut self, path: &Path<'a>, v: MRecord) {
        self._set_path(path.0.iter(), v);
    }

    fn _del_path<'a>(&mut self, prev: &'a PathStep<'a>, mut path: impl Iterator<Item = &'a PathStep<'a>>) -> MRecord {
        let mut n = self.0.lock().unwrap();
        let n = (*n).convert_r_mut(|r| {
            return (*r.0).clone().map(MRecord::wrap);
        });
        match path.next() {
            Some(step) => {
                return n.get_rstep_fill(prev)._del_path(step, path);
            }
            None => {
                return n.del_rpart(prev);
            }
        }
    }

    pub fn del_path(&mut self, path: &str) -> MRecord {
        return self.del_path_obj(&Path::new(path));
    }

    pub fn del_path_obj<'a>(&mut self, path: &Path<'a>) -> MRecord {
        let mut path = path.0.iter();
        if let Some(first) = path.next() {
            return self._del_path(first, path);
        }
        panic!();
    }
}
