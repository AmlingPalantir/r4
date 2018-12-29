extern crate misc;

mod float;
pub use float::F64HashDishonorProxy;
pub use float::F64SortDishonorProxy;

mod record;
pub use record::Record;

mod mrecord;
pub use mrecord::MRecord;

#[cfg(test)]
mod tests;

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

    fn coerce_num(&self) -> Either<i64, f64> {
        match self.maybe_primitive() {
            Some(JsonPrimitive::Null()) => {
                return Either::Left(0);
            }
            Some(JsonPrimitive::NumberI64(n)) => {
                return Either::Left(n);
            }
            Some(JsonPrimitive::NumberF64(ref n)) => {
                return Either::Right(n.0);
            }
            Some(JsonPrimitive::String(s)) => {
                if let Ok(n) = s.parse() {
                    return Either::Left(n);
                }
                if let Ok(n) = s.parse() {
                    return Either::Right(n);
                }
                panic!();
            }
            _ => {
                panic!();
            }
        }
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

    fn expect_string(&self) -> Arc<str> {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::String(ref s)) => s.clone(),
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
