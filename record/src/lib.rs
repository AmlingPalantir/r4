#[derive(Clone)]
pub struct Record(Arc<JsonPart>);

use std::collections::BTreeMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Clone)]
enum JsonPart {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(Arc<str>),
    Array(Vec<Record>),
    Hash(BTreeMap<Arc<str>, Record>),
}

pub trait FromPrimitive<P> {
    fn from_primitive(p: P) -> Self;
}

impl FromPrimitive<u32> for Record {
    fn from_primitive(n: u32) -> Self {
        return Record(Arc::new(JsonPart::Number(serde_json::Number::from(n))));
    }
}

impl Record {
    pub fn null() -> Self {
        return Record(Arc::new(JsonPart::Null));
    }

    pub fn from_primitive_string<S: Deref<Target = str>>(s: S) -> Self {
        return Record(Arc::new(JsonPart::String(Arc::from(&*s))));
    }

    pub fn from_vec(arr: Vec<Record>) -> Self {
        return Record(Arc::new(JsonPart::Array(arr)));
    }

    pub fn get_hash(&self, key: &str) -> Option<Record> {
        if let JsonPart::Null = *self.0 {
            return None;
        }
        if let JsonPart::Hash(ref map) = *self.0 {
            return match map.get(key) {
                Some(r) => Some(r.clone()),
                None => None,
            };
        }
        panic!();
    }

    pub fn get_array(&self, key: usize) -> Option<Record> {
        if let JsonPart::Null = *self.0 {
            return None;
        }
        if let JsonPart::Array(ref arr) = *self.0 {
            if key >= arr.len() {
                return None;
            }
            return Some(arr[key].clone());
        }
        panic!();
    }


    pub fn get_path(&self, path: &str) -> Record {
        return path.split('/').fold(Some(self.clone()), |r, part| {
            match r {
                Some(r) => {
                    if part.starts_with('#') {
                        return r.get_array(part[1..].parse().unwrap())
                    }
                    return r.get_hash(part);
                }
                None => {
                    return None;
                }
            }
        }).unwrap_or_else(Record::null);
    }

    fn get_path_mut(&mut self, path: &str) -> &mut JsonPart {
        fn _get_hash_mut<'a>(r: &'a mut JsonPart, key: &str) -> &'a mut JsonPart {
            if let JsonPart::Null = *r {
                *r = JsonPart::Hash(BTreeMap::new());
            }
            if let JsonPart::Hash(ref mut map) = *r {
                return Arc::make_mut(&mut map.entry(Arc::from(key)).or_insert(Record(Arc::new(JsonPart::Null))).0);
            }
            panic!();
        }

        fn _get_array_mut(r: &mut JsonPart, key: usize) -> &mut JsonPart {
            if let JsonPart::Null = *r {
                *r = JsonPart::Array(Vec::new());
            }
            if let JsonPart::Array(ref mut arr) = *r {
                while key >= arr.len() {
                    arr.push(Record(Arc::new(JsonPart::Null)));
                }
                return Arc::make_mut(&mut arr[key].0);
            }
            panic!();
        }

        return path.split('/').fold(Arc::make_mut(&mut self.0), |r, part| {
            if part.starts_with('#') {
                return _get_array_mut(r, part[1..].parse().unwrap());
            }
            return _get_hash_mut(r, part);
        });
    }

    pub fn set_path(&mut self, path: &str, r: Record) {
        *self.get_path_mut(path) = (*r.0).clone();
    }
}

impl FromStr for Record {
    type Err = serde_json::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn convert_part(p: &serde_json::value::Value) -> Record {
            return Record(Arc::new(match p {
                serde_json::value::Value::Null => JsonPart::Null,
                serde_json::value::Value::Bool(b) => JsonPart::Bool(*b),
                serde_json::value::Value::Number(n) => JsonPart::Number(n.clone()),
                serde_json::value::Value::String(s) => JsonPart::String(Arc::from(s.clone())),
                serde_json::value::Value::Array(arr) => JsonPart::Array(arr.iter().map(|v| convert_part(v)).collect()),
                serde_json::value::Value::Object(map) => JsonPart::Hash(map.iter().map(|(k, v)| (Arc::from(k.clone()), convert_part(v))).collect()),
            }));
        }

        return serde_json::from_str(s).map(|j| convert_part(&j));
    }
}

impl ToString for Record {
    fn to_string(&self) -> String {
        fn _to_string_aux(p: &Record, acc: &mut String) {
            match &*p.0 {
                JsonPart::Null => {
                    acc.push_str("null");
                }
                JsonPart::Bool(b) => {
                    acc.push_str(if *b { "true" } else { "false" });
                }
                JsonPart::Number(n) => {
                    acc.push_str(&serde_json::to_string(&n).unwrap());
                }
                JsonPart::String(s) => {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert!(Record::from_str("x").is_err());
    }

    #[test]
    fn test2() {
        assert!(!Record::from_str("{}").is_err());
    }

    #[test]
    fn test_serde() {
        let s = "{\"x\":[{\"y\":\"z\"}]}";
        let r = Record::from_str(s).unwrap();
        assert_eq!(r.to_string(), s);
    }

    #[test]
    fn test_get_path() {
        let r = Record::from_str("{\"x\":[{\"y\":\"z\"}]}").unwrap();
        assert_eq!(r.get_path("x").to_string(), "[{\"y\":\"z\"}]");
        assert_eq!(r.get_path("y").to_string(), "null");
        assert_eq!(r.get_path("y/z").to_string(), "null");
        assert_eq!(r.get_path("x/#0").to_string(), "{\"y\":\"z\"}");
        assert_eq!(r.get_path("x/#1").to_string(), "null");
        assert_eq!(r.get_path("x/#0/y").to_string(), "\"z\"");
    }

    #[test]
    fn test_set_path() {
        let mut r = Record::from_str("{\"x\":[{\"y\":\"z\"}]}").unwrap();
        let r2 = r.clone();
        r.set_path("x/#0/y", Record::from_primitive_string("w"));
        assert_eq!(r.to_string(), "{\"x\":[{\"y\":\"w\"}]}");
        assert_eq!(r2.to_string(), "{\"x\":[{\"y\":\"z\"}]}");
        r.set_path("a/#2/b", Record::from_primitive_string("c"));
        assert_eq!(r.to_string(), "{\"a\":[null,null,{\"b\":\"c\"}],\"x\":[{\"y\":\"w\"}]}");
    }
}
