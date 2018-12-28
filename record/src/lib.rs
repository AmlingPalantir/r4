extern crate misc;

pub mod float;

#[cfg(test)]
mod tests;

use float::F64HashDishonorProxy;
use misc::Either;
use std::collections::BTreeMap;
use std::sync::Arc;
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

}

impl<T: RecordTrait> RecordNode<T> {
    fn get_rstep(&self, step: &RefPathStep) -> Option<&T> {
        match step {
            RefPathStep::Hash(s) => {
                if let RecordNode::Hash(hash) = self {
                    return hash.get(*s);
                }
                panic!("hash step on non-hash");
            }
            RefPathStep::Array(n) => {
                if let RecordNode::Array(arr) = self {
                    return arr.get(*n);
                }
                panic!("array step on non-array");
            }
        }
    }

    fn get_rstep_mut(&mut self, step: &RefPathStep) -> Option<&mut T> {
        match step {
            RefPathStep::Hash(s) => {
                if let RecordNode::Hash(hash) = self {
                    return hash.get_mut(*s);
                }
                panic!("hash step on non-hash");
            }
            RefPathStep::Array(n) => {
                if let RecordNode::Array(arr) = self {
                    return arr.get_mut(*n);
                }
                panic!("array step on non-array");
            }
        }
    }

    fn get_rstep_fill(&mut self, step: &RefPathStep) -> &mut T {
        match step {
            RefPathStep::Hash(s) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Hash(BTreeMap::new());
                }
                if let RecordNode::Hash(hash) = self {
                    return hash.entry(Arc::from(*s)).or_insert_with(|| T::new(RecordNode::Primitive(JsonPrimitive::Null())));
                }
                panic!();
            }
            RefPathStep::Array(n) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Array(Vec::new());
                }
                if let RecordNode::Array(arr) = self {
                    while *n >= arr.len() {
                        arr.push(T::new(RecordNode::Primitive(JsonPrimitive::Null())));
                    }
                    return &mut arr[*n];
                }
                panic!();
            }
        }
    }

    fn del_rpart(&mut self, step: &RefPathStep) -> T {
        unimplemented!();
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
        return RefPath::new(path).0.iter().fold(Some(self), |r, part| {
            return match r {
                Some(r) => r.0.get_rstep(part),
                None => None,
            };
        });
    }

    pub fn set_path(&mut self, path: &str, v: Record) {
        *RefPath::new(path).0.iter().fold(self, |r, s| Arc::make_mut(&mut r.0).get_rstep_fill(s)) = v;
    }

    pub fn del_path(&mut self, path: &str) -> Record {
        let path = RefPath::new(path);
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

    pub fn coerce_f64(&self) -> f64 {
        return match *self.0 {
            RecordNode::Primitive(JsonPrimitive::NumberF64(ref f)) => f.0,
            RecordNode::Primitive(JsonPrimitive::NumberI64(i)) => i as f64,
            RecordNode::Primitive(JsonPrimitive::String(ref s)) => s.parse().unwrap(),
            _ => panic!(),
        };
    }

    pub fn coerce_string(&self) -> Arc<str> {
        return match *self.0 {
            RecordNode::Primitive(JsonPrimitive::Null()) => Arc::from(""),
            RecordNode::Primitive(JsonPrimitive::Bool(b)) => Arc::from(b.to_string()),
            RecordNode::Primitive(JsonPrimitive::NumberF64(ref f)) => Arc::from(f.0.to_string()),
            RecordNode::Primitive(JsonPrimitive::NumberI64(i)) => Arc::from(i.to_string()),
            RecordNode::Primitive(JsonPrimitive::String(ref s)) => s.clone(),
            _ => panic!(),
        };
    }

    pub fn coerce_bool(&self) -> bool {
        return match *self.0 {
            RecordNode::Primitive(JsonPrimitive::Null()) => false,
            RecordNode::Primitive(JsonPrimitive::Bool(b)) => b,
            RecordNode::Primitive(JsonPrimitive::NumberF64(ref f)) => f.0 != 0.0,
            RecordNode::Primitive(JsonPrimitive::NumberI64(i)) => i != 0,
            RecordNode::Primitive(JsonPrimitive::String(ref s)) => !s.is_empty(),
            RecordNode::Array(_) => true,
            RecordNode::Hash(_) => true,
        }
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

    pub fn maybe_i64(&self) -> Option<i64> {
        return match *self.0 {
            RecordNode::Primitive(JsonPrimitive::NumberI64(n)) => Some(n),
            _ => None,
        }
    }

    pub fn maybe_num(&self) -> Option<f64> {
        return match *self.0 {
            RecordNode::Primitive(JsonPrimitive::NumberI64(n)) => Some(n as f64),
            RecordNode::Primitive(JsonPrimitive::NumberF64(ref n)) => Some(n.0),
            _ => None,
        }
    }
}

enum RefPathStep<'a> {
    Hash(&'a str),
    Array(usize),
}

struct RefPath<'a>(Vec<RefPathStep<'a>>);

impl<'a> RefPath<'a> {
    fn new(s: &'a str) -> RefPath<'a> {
        return RefPath(s.split('/').map(|e| {
            if e.starts_with('#') {
                return RefPathStep::Array(e[1..].parse().unwrap());
            }
            return RefPathStep::Hash(e);
        }).collect());
    }

    fn to_owned(&self) -> OwnPath {
        return OwnPath(self.0.iter().map(|e| {
            return match e {
                RefPathStep::Hash(s) => OwnPathStep::Hash(s.to_string()),
                RefPathStep::Array(n) => OwnPathStep::Array(*n),
            };
        }).collect());
    }
}

enum OwnPathStep {
    Hash(String),
    Array(usize),
}

struct OwnPath(Vec<OwnPathStep>);
