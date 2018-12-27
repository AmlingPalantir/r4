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
pub enum JsonPart {
    Primitive(JsonPrimitive),
    Array(Vec<Record>),
    Hash(BTreeMap<Arc<str>, Record>),
}

impl JsonPart {
    fn get_hash_mut(&mut self, key: &str) -> &mut JsonPart {
        if let JsonPart::Primitive(JsonPrimitive::Null()) = *self {
            *self = JsonPart::Hash(BTreeMap::new());
        }
        if let JsonPart::Hash(ref mut map) = *self {
            return Arc::make_mut(&mut map.entry(Arc::from(key)).or_insert(Record::null()).0);
        }
        panic!("JsonPart::get_hash_mut() on non-hash");
    }

    fn get_array_mut(&mut self, key: usize) -> &mut JsonPart {
        if let JsonPart::Primitive(JsonPrimitive::Null()) = *self {
            *self = JsonPart::Array(Vec::new());
        }
        if let JsonPart::Array(ref mut arr) = *self {
            while key >= arr.len() {
                arr.push(Record::null());
            }
            return Arc::make_mut(&mut arr[key].0);
        }
        panic!("JsonPart::get_array_mut() on non-array");
    }

    fn get_rpart_mut(&mut self, part: &Either<&str, usize>) -> &mut JsonPart {
        return match part {
            Either::Left(s) => self.get_hash_mut(s),
            Either::Right(idx) => self.get_array_mut(*idx),
        };
    }

    fn del_rpart(&mut self, part: &Either<&str, usize>) -> Record {
        match *part {
            Either::Left(s) => {
                if let JsonPart::Primitive(JsonPrimitive::Null()) = *self {
                    *self = JsonPart::Hash(BTreeMap::new());
                }
                if let JsonPart::Hash(ref mut map) = *self {
                    return map.remove(&Arc::from(s)).unwrap_or_else(Record::null);
                }
                panic!();
            }
            Either::Right(_idx) => {
                panic!();
            }
        }
    }
}

#[derive(Clone)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub struct Record(pub Arc<JsonPart>);

impl Record {
    pub fn null() -> Self {
        return Record::from(JsonPrimitive::Null());
    }

    pub fn empty_hash() -> Self {
        return Record(Arc::new(JsonPart::Hash(BTreeMap::new())));
    }

    pub fn from_vec(arr: Vec<Record>) -> Self {
        return Record(Arc::new(JsonPart::Array(arr)));
    }

    pub fn from_hash(hash: BTreeMap<Arc<str>, Record>) -> Self {
        return Record(Arc::new(JsonPart::Hash(hash)));
    }

    pub fn get_hash(&self, key: &str) -> Option<Record> {
        if let JsonPart::Primitive(JsonPrimitive::Null()) = *self.0 {
            return None;
        }
        if let JsonPart::Hash(ref map) = *self.0 {
            return match map.get(key) {
                Some(r) => Some(r.clone()),
                None => None,
            };
        }
        panic!("Record::get_hash() on non-hash");
    }

    pub fn get_array(&self, key: usize) -> Option<Record> {
        if let JsonPart::Primitive(JsonPrimitive::Null()) = *self.0 {
            return None;
        }
        if let JsonPart::Array(ref arr) = *self.0 {
            if key >= arr.len() {
                return None;
            }
            return Some(arr[key].clone());
        }
        panic!("Record::get_array() on non-array");
    }

    pub fn has_path(&self, path: &str) -> bool {
        return self.get_path_opt(path).is_some();
    }

    pub fn get_path(&self, path: &str) -> Record {
        return self.get_path_opt(path).unwrap_or_else(Record::null);
    }

    pub fn get_rpart(&self, part: &Either<&str, usize>) -> Option<Record> {
        return match part {
            Either::Left(s) => self.get_hash(s),
            Either::Right(idx) => self.get_array(*idx),
        };
    }

    pub fn get_path_opt(&self, path: &str) -> Option<Record> {
        return RefPath::new(path).0.iter().fold(Some(self.clone()), |r, part| {
            return match r {
                Some(r) => r.get_rpart(part),
                None => None,
            };
        });
    }

    fn get_path_mut(&mut self, path: &str) -> &mut JsonPart {
        return RefPath::new(path).0.iter().fold(Arc::make_mut(&mut self.0), JsonPart::get_rpart_mut);
    }

    pub fn set_path(&mut self, path: &str, r: Record) {
        *self.get_path_mut(path) = (*r.0).clone();
    }

    pub fn del_path(&mut self, path: &str) -> Record {
        let path = RefPath::new(path);
        let x = path.0.iter().fold(
            Either::Left(Arc::make_mut(&mut self.0)),
            |e, part| {
                match e {
                    Either::Left(r) => {
                        return Either::Right((r, part));
                    }
                    Either::Right((r, prev)) => {
                        let r = r.get_rpart_mut(prev);
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
                return r.del_rpart(part);
            }
        }
    }

    pub fn parse(s: &str) -> Self {
        fn convert_part(p: &serde_json::value::Value) -> Record {
            return Record(Arc::new(match p {
                serde_json::value::Value::Null => JsonPart::Primitive(JsonPrimitive::Null()),
                serde_json::value::Value::Bool(b) => JsonPart::Primitive(JsonPrimitive::Bool(*b)),
                serde_json::value::Value::Number(n) => JsonPart::Primitive(JsonPrimitive::from_serde_number(n)),
                serde_json::value::Value::String(s) => JsonPart::Primitive(JsonPrimitive::String(Arc::from(s.clone()))),
                serde_json::value::Value::Array(arr) => JsonPart::Array(arr.iter().map(|v| convert_part(v)).collect()),
                serde_json::value::Value::Object(map) => JsonPart::Hash(map.iter().map(|(k, v)| (Arc::from(k.clone()), convert_part(v))).collect()),
            }));
        }

        return convert_part(&serde_json::from_str(s).unwrap());
    }

    pub fn deparse(&self) -> String {
        fn _to_string_aux(p: &Record, acc: &mut String) {
            match &*p.0 {
                JsonPart::Primitive(JsonPrimitive::Null()) => {
                    acc.push_str("null");
                }
                JsonPart::Primitive(JsonPrimitive::Bool(b)) => {
                    acc.push_str(if *b { "true" } else { "false" });
                }
                JsonPart::Primitive(JsonPrimitive::NumberI64(n)) => {
                    acc.push_str(&serde_json::to_string(&serde_json::Number::from(*n)).unwrap());
                }
                JsonPart::Primitive(JsonPrimitive::NumberF64(n)) => {
                    acc.push_str(&n.to_json_string());
                }
                JsonPart::Primitive(JsonPrimitive::String(s)) => {
                    let sr: &str = &*s;
                    acc.push_str(&serde_json::to_string(sr).unwrap());
                }
                JsonPart::Array(arr) => {
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
                JsonPart::Hash(map) => {
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
            JsonPart::Primitive(JsonPrimitive::String(ref s)) => s.clone(),
            _ => panic!(),
        };
    }

    pub fn coerce_f64(&self) -> f64 {
        return match *self.0 {
            JsonPart::Primitive(JsonPrimitive::NumberF64(ref f)) => f.0,
            JsonPart::Primitive(JsonPrimitive::NumberI64(i)) => i as f64,
            JsonPart::Primitive(JsonPrimitive::String(ref s)) => s.parse().unwrap(),
            _ => panic!(),
        };
    }

    pub fn coerce_string(&self) -> Arc<str> {
        return match *self.0 {
            JsonPart::Primitive(JsonPrimitive::Null()) => Arc::from(""),
            JsonPart::Primitive(JsonPrimitive::Bool(b)) => Arc::from(b.to_string()),
            JsonPart::Primitive(JsonPrimitive::NumberF64(ref f)) => Arc::from(f.0.to_string()),
            JsonPart::Primitive(JsonPrimitive::NumberI64(i)) => Arc::from(i.to_string()),
            JsonPart::Primitive(JsonPrimitive::String(ref s)) => s.clone(),
            _ => panic!(),
        };
    }

    pub fn coerce_bool(&self) -> bool {
        return match *self.0 {
            JsonPart::Primitive(JsonPrimitive::Null()) => false,
            JsonPart::Primitive(JsonPrimitive::Bool(b)) => b,
            JsonPart::Primitive(JsonPrimitive::NumberF64(ref f)) => f.0 != 0.0,
            JsonPart::Primitive(JsonPrimitive::NumberI64(i)) => i != 0,
            JsonPart::Primitive(JsonPrimitive::String(ref s)) => !s.is_empty(),
            JsonPart::Array(_) => true,
            JsonPart::Hash(_) => true,
        }
    }

    pub fn expect_array(&self) -> &Vec<Record> {
        return match *self.0 {
            JsonPart::Array(ref arr) => arr,
            _ => panic!(),
        };
    }

    pub fn expect_hash(&self) -> &BTreeMap<Arc<str>, Record> {
        return match *self.0 {
            JsonPart::Hash(ref hash) => hash,
            _ => panic!(),
        };
    }

    pub fn pretty_string(&self) -> String {
        return match *self.0 {
            JsonPart::Primitive(JsonPrimitive::String(ref s)) => s.to_string(),
            _ => self.deparse(),
        };
    }

    pub fn maybe_i64(&self) -> Option<i64> {
        return match *self.0 {
            JsonPart::Primitive(JsonPrimitive::NumberI64(n)) => Some(n),
            _ => None,
        }
    }

    pub fn maybe_num(&self) -> Option<f64> {
        return match *self.0 {
            JsonPart::Primitive(JsonPrimitive::NumberI64(n)) => Some(n as f64),
            JsonPart::Primitive(JsonPrimitive::NumberF64(ref n)) => Some(n.0),
            _ => None,
        }
    }
}

impl<T> From<T> for Record where JsonPrimitive: From<T> {
    fn from(t: T) -> Self {
        return Record(Arc::new(JsonPart::Primitive(JsonPrimitive::from(t))));
    }
}

pub struct RefPath<'a>(pub Vec<Either<&'a str, usize>>);

impl<'a> RefPath<'a> {
    pub fn new(s: &'a str) -> RefPath<'a> {
        return RefPath(s.split('/').map(|e| {
            if e.starts_with('#') {
                return Either::Right(e[1..].parse().unwrap());
            }
            return Either::Left(e);
        }).collect());
    }

    pub fn to_owned(&self) -> OwnPath {
        return OwnPath(self.0.iter().map(|e| e.clone().map_left(|s| s.to_string())).collect());
    }
}

pub struct OwnPath(pub Vec<Either<String, usize>>);
