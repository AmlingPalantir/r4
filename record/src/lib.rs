extern crate misc;

pub mod float;

#[cfg(test)]
mod tests;

use float::F64HashDishonorProxy;
use misc::Either;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::vec::Vec;

#[derive(Clone)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub enum JsonPrimitive {
    Null(),
    Bool(bool),
    NumberI64(i64),
    NumberF64(F64HashDishonorProxy),
    String(Arc<str>),
}

impl JsonPrimitive {
    fn from_serde_number(n: &serde_json::Number) -> JsonPrimitive {
        if let Some(n) = n.as_i64() {
            return JsonPrimitive::NumberI64(n);
        }
        if let Some(n) = n.as_f64() {
            return JsonPrimitive::NumberF64(F64HashDishonorProxy(n));
        }
        panic!("Unhandled JSON number type: {}", n);
    }
}

impl From<bool> for JsonPrimitive {
    fn from(b: bool) -> Self {
        return JsonPrimitive::Bool(b);
    }
}

impl From<i64> for JsonPrimitive {
    fn from(n: i64) -> Self {
        return JsonPrimitive::NumberI64(n);
    }
}

impl From<f64> for JsonPrimitive {
    fn from(n: f64) -> Self {
        return JsonPrimitive::NumberF64(F64HashDishonorProxy(n));
    }
}

impl From<Arc<str>> for JsonPrimitive {
    fn from(s: Arc<str>) -> Self {
        return JsonPrimitive::String(s);
    }
}

impl From<String> for JsonPrimitive {
    fn from(s: String) -> Self {
        return JsonPrimitive::String(Arc::from(s));
    }
}

impl<'a> From<&'a str> for JsonPrimitive {
    fn from(s: &'a str) -> Self {
        return JsonPrimitive::String(Arc::from(s));
    }
}



#[derive(Clone)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub enum RecordNode<T> {
    Primitive(JsonPrimitive),
    Array(Vec<T>),
    Hash(BTreeMap<Arc<str>, T>),
}

impl<T, F> From<F> for RecordNode<T> where JsonPrimitive: From<F> {
    fn from(f: F) -> Self {
        return RecordNode::Primitive(JsonPrimitive::from(f));
    }
}

impl<T> RecordNode<T> {
    fn map<S, F: Fn(T) -> S>(self, f: F) -> RecordNode<S> {
        return match self {
            RecordNode::Primitive(p) => RecordNode::Primitive(p),
            RecordNode::Array(arr) => RecordNode::Array(arr.into_iter().map(f).collect()),
            RecordNode::Hash(hash) => RecordNode::Hash(hash.into_iter().map(|(k, v)| (k, f(v))).collect()),
        };
    }

    fn maybe_primitive(&self) -> Option<JsonPrimitive> {
        return match self {
            RecordNode::Primitive(p) => Some(p.clone()),
            _ => None,
        };
    }
}

pub trait RecordTrait: std::marker::Sized {
    fn new(RecordNode<Self>) -> Self;

    fn null() -> Self {
        return Self::new(RecordNode::from(JsonPrimitive::Null()));
    }

    fn empty_hash() -> Self {
        return Self::from_hash(BTreeMap::new());
    }

    fn from_vec(arr: Vec<Self>) -> Self {
        return Self::new(RecordNode::Array(arr));
    }

    fn from_hash(hash: BTreeMap<Arc<str>, Self>) -> Self {
        return Self::new(RecordNode::Hash(hash));
    }

    fn maybe_primitive(&self) -> Option<JsonPrimitive>;

    fn maybe_i64(&self) -> Option<i64> {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::NumberI64(n)) => Some(n),
            _ => None,
        };
    }

    fn maybe_num(&self) -> Option<f64> {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::NumberI64(n)) => Some(n as f64),
            Some(JsonPrimitive::NumberF64(ref n)) => Some(n.0),
            _ => None,
        };
    }

    fn coerce_string(&self) -> Arc<str> {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::Null()) => Arc::from(""),
            Some(JsonPrimitive::Bool(b)) => Arc::from(b.to_string()),
            Some(JsonPrimitive::NumberF64(ref f)) => Arc::from(f.0.to_string()),
            Some(JsonPrimitive::NumberI64(i)) => Arc::from(i.to_string()),
            Some(JsonPrimitive::String(ref s)) => s.clone(),
            _ => panic!(),
        };
    }

    fn coerce_bool(&self) -> bool {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::Null()) => false,
            Some(JsonPrimitive::Bool(b)) => b,
            Some(JsonPrimitive::NumberF64(ref f)) => f.0 != 0.0,
            Some(JsonPrimitive::NumberI64(i)) => i != 0,
            Some(JsonPrimitive::String(ref s)) => !s.is_empty(),
            None => true,
        };
    }

    fn coerce_f64(&self) -> f64 {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::NumberF64(ref f)) => f.0,
            Some(JsonPrimitive::NumberI64(i)) => i as f64,
            Some(JsonPrimitive::String(ref s)) => s.parse().unwrap(),
            _ => panic!(),
        };
    }
}

impl<T: RecordTrait> RecordNode<T> {
    fn get_rstep(&self, step: &PathStep) -> Option<&T> {
        match step {
            PathStep::RefHash(s) => {
                if let RecordNode::Hash(hash) = self {
                    return hash.get(*s);
                }
                panic!("hash step on non-hash");
            }
            PathStep::OwnHash(s) => {
                if let RecordNode::Hash(hash) = self {
                    return hash.get(s);
                }
                panic!("hash step on non-hash");
            }
            PathStep::Array(n) => {
                if let RecordNode::Array(arr) = self {
                    return arr.get(*n);
                }
                panic!("array step on non-array");
            }
        }
    }

    fn get_rstep_mut(&mut self, step: &PathStep) -> Option<&mut T> {
        match step {
            PathStep::RefHash(s) => {
                if let RecordNode::Hash(hash) = self {
                    return hash.get_mut(*s);
                }
                panic!("hash step on non-hash");
            }
            PathStep::OwnHash(s) => {
                if let RecordNode::Hash(hash) = self {
                    return hash.get_mut(s);
                }
                panic!("hash step on non-hash");
            }
            PathStep::Array(n) => {
                if let RecordNode::Array(arr) = self {
                    return arr.get_mut(*n);
                }
                panic!("array step on non-array");
            }
        }
    }

    fn get_rstep_fill(&mut self, step: &PathStep) -> &mut T {
        match step {
            PathStep::RefHash(s) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Hash(BTreeMap::new());
                }
                if let RecordNode::Hash(hash) = self {
                    return hash.entry(Arc::from(*s)).or_insert_with(T::null);
                }
                panic!();
            }
            PathStep::OwnHash(s) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Hash(BTreeMap::new());
                }
                if let RecordNode::Hash(hash) = self {
                    return hash.entry(s.clone()).or_insert_with(T::null);
                }
                panic!();
            }
            PathStep::Array(n) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Array(Vec::new());
                }
                if let RecordNode::Array(arr) = self {
                    while *n >= arr.len() {
                        arr.push(T::null());
                    }
                    return &mut arr[*n];
                }
                panic!();
            }
        }
    }

    fn del_rpart(&mut self, step: &PathStep) -> T {
        match step {
            PathStep::RefHash(s) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Hash(BTreeMap::new());
                }
                if let RecordNode::Hash(hash) = self {
                    return hash.remove(s as &str).unwrap_or_else(T::null);
                }
                panic!();
            }
            PathStep::OwnHash(s) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Hash(BTreeMap::new());
                }
                if let RecordNode::Hash(hash) = self {
                    return hash.remove(s).unwrap_or_else(T::null);
                }
                panic!();
            }
            PathStep::Array(_n) => {
                panic!();
            }
        }
    }
}



enum PathStep<'a> {
    RefHash(&'a str),
    OwnHash(Arc<str>),
    Array(usize),
}

pub struct Path<'a>(Vec<PathStep<'a>>);
pub type OwnPath = Path<'static>;

impl<'a> Path<'a> {
    pub fn new(s: &'a str) -> Path<'a> {
        return Path(s.split('/').map(|e| {
            if e.starts_with('#') {
                return PathStep::Array(e[1..].parse().unwrap());
            }
            return PathStep::RefHash(e);
        }).collect());
    }

    pub fn to_owned(self) -> OwnPath {
        return Path(self.0.into_iter().map(|e| {
            return match e {
                PathStep::RefHash(s) => PathStep::OwnHash(Arc::from(s)),
                PathStep::OwnHash(s) => PathStep::OwnHash(Arc::from(s)),
                PathStep::Array(n) => PathStep::Array(n),
            };
        }).collect());
    }
}



#[derive(Clone)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub struct Record(Arc<RecordNode<Record>>);

impl RecordTrait for Record {
    fn new(n: RecordNode<Self>) -> Self {
        return Record(Arc::new(n));
    }

    fn maybe_primitive(&self) -> Option<JsonPrimitive> {
        return self.0.maybe_primitive();
    }
}

impl<T> From<T> for Record where RecordNode<Record>: From<T> {
    fn from(t: T) -> Self {
        return Record::new(RecordNode::from(t));
    }
}

impl Record {
    pub fn has_path(&self, path: &str) -> bool {
        return self.get_path_opt(path).is_some();
    }

    pub fn get_path(&self, path: &str) -> Record {
        return self.get_path_opt(path).map(Record::clone).unwrap_or_else(Record::null);
    }

    fn get_path_opt(&self, path: &str) -> Option<&Record> {
        return Path::new(path).0.iter().fold(Some(self), |r, part| {
            return match r {
                Some(r) => r.0.get_rstep(part),
                None => None,
            };
        });
    }

    pub fn set_path(&mut self, path: &str, v: Record) {
        *Path::new(path).0.iter().fold(self, |r, s| Arc::make_mut(&mut r.0).get_rstep_fill(s)) = v;
    }

    pub fn del_path(&mut self, path: &str) -> Record {
        let path = Path::new(path);
        let x = path.0.iter().fold(
            Either::Left(self),
            |e, part| {
                match e {
                    Either::Left(r) => {
                        return Either::Right((r, part));
                    }
                    Either::Right((r, prev)) => {
                        let r = Arc::make_mut(&mut r.0).get_rstep_fill(prev);
                        return Either::Right((r, part));
                    }
                }
            }
        );
        match x {
            Either::Left(_) => {
                panic!();
            }
            Either::Right((r, part)) => {
                return Arc::make_mut(&mut r.0).del_rpart(part);
            }
        }
    }

    pub fn parse(s: &str) -> Self {
        fn convert_part(p: &serde_json::value::Value) -> Record {
            return match p {
                serde_json::value::Value::Null => Record::null(),
                serde_json::value::Value::Bool(b) => Record::from(*b),
                serde_json::value::Value::Number(n) => Record::from(JsonPrimitive::from_serde_number(n)),
                serde_json::value::Value::String(s) => Record::from(s.clone()),
                serde_json::value::Value::Array(arr) => Record::from_vec(arr.iter().map(|v| convert_part(v)).collect()),
                serde_json::value::Value::Object(map) => Record::from_hash(map.iter().map(|(k, v)| (Arc::from(k.clone()), convert_part(v))).collect()),
            };
        }

        return convert_part(&serde_json::from_str(s).unwrap());
    }

    pub fn deparse(&self) -> String {
        fn _to_string_aux(p: &Record, acc: &mut String) {
            match &*p.0 {
                RecordNode::Primitive(JsonPrimitive::Null()) => {
                    acc.push_str("null");
                }
                RecordNode::Primitive(JsonPrimitive::Bool(b)) => {
                    acc.push_str(if *b { "true" } else { "false" });
                }
                RecordNode::Primitive(JsonPrimitive::NumberI64(n)) => {
                    acc.push_str(&serde_json::to_string(&serde_json::Number::from(*n)).unwrap());
                }
                RecordNode::Primitive(JsonPrimitive::NumberF64(n)) => {
                    acc.push_str(&n.to_json_string());
                }
                RecordNode::Primitive(JsonPrimitive::String(s)) => {
                    let sr: &str = &*s;
                    acc.push_str(&serde_json::to_string(sr).unwrap());
                }
                RecordNode::Array(arr) => {
                    acc.push_str("[");
                    for e in arr.iter().enumerate() {
                        let (i, v) = e;
                        if i > 0 {
                            acc.push_str(",");
                        }
                        _to_string_aux(v, acc);
                    }
                    acc.push_str("]");
                }
                RecordNode::Hash(map) => {
                    acc.push_str("{");
                    for e in map.iter().enumerate() {
                        let (i, (k, v)) = e;
                        if i > 0 {
                            acc.push_str(",");
                        }
                        let kr: &str = &*k;
                        acc.push_str(&serde_json::to_string(kr).unwrap());
                        acc.push_str(":");
                        _to_string_aux(v, acc);
                    }
                    acc.push_str("}");
                }
            }
        }

        let mut ret = String::new();
        _to_string_aux(self, &mut ret);
        return ret;
    }

    pub fn expect_string(&self) -> Arc<str> {
        return match *self.0 {
            RecordNode::Primitive(JsonPrimitive::String(ref s)) => s.clone(),
            _ => panic!(),
        };
    }

    pub fn expect_array(&self) -> &Vec<Record> {
        return match *self.0 {
            RecordNode::Array(ref arr) => arr,
            _ => panic!(),
        };
    }

    pub fn expect_hash(&self) -> &BTreeMap<Arc<str>, Record> {
        return match *self.0 {
            RecordNode::Hash(ref hash) => hash,
            _ => panic!(),
        };
    }

    pub fn pretty_string(&self) -> String {
        return match *self.0 {
            RecordNode::Primitive(JsonPrimitive::String(ref s)) => s.to_string(),
            _ => self.deparse(),
        };
    }
}

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
        return self._get_path(Path::new(path).0.iter());
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
        return self._get_path_fill(Path::new(path).0.iter());
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
        self._set_path(Path::new(path).0.iter(), v);
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
        let path = Path::new(path);
        let mut path = path.0.iter();
        if let Some(first) = path.next() {
            return self._del_path(first, path);
        }
        panic!();
    }
}
