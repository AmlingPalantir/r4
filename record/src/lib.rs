#[derive(Clone)]
struct Record(Arc<JsonPart>);

use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Clone)]
enum JsonPart {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(Arc<str>),
    // TODO: less crummy Vec (better splice?)
    Array(Vec<Arc<JsonPart>>),
    Hash(BTreeMap<Arc<str>, Arc<JsonPart>>),
}

impl Record {
    fn null() -> Self {
        return Record(Arc::new(JsonPart::Null));
    }

    fn get_hash(&self, key: Arc<str>) -> Option<Record> {
        if let JsonPart::Null = *self.0 {
            return None;
        }
        if let JsonPart::Hash(ref map) = *self.0 {
            return match map.get(&key) {
                Some(arc) => Some(Record(arc.clone())),
                None => None,
            };
        }
        panic!();
    }

    fn get_array(&self, key: usize) -> Option<Record> {
        if let JsonPart::Null = *self.0 {
            return None;
        }
        if let JsonPart::Array(ref arr) = *self.0 {
            if key >= arr.len() {
                return None;
            }
            return Some(Record(arr[key].clone()));
        }
        panic!();
    }


    fn get_path(&self, path: Arc<str>) -> Record {
        return path.split('/').fold(Some(self.clone()), |r, part| {
            match r {
                Some(r) => {
                    if part.starts_with('#') {
                        return r.get_array(part[1..].parse().unwrap())
                    }
                    return r.get_hash(Arc::from(part));
                }
                None => {
                    return None;
                }
            }
        }).unwrap_or_else(Record::null);
    }

    fn get_path_mut(&mut self, path: Arc<str>) -> &mut JsonPart {
        fn _get_hash_mut(r: &mut JsonPart, key: Arc<str>) -> &mut JsonPart {
            if let JsonPart::Null = *r {
                *r = JsonPart::Hash(BTreeMap::new());
            }
            if let JsonPart::Hash(ref mut map) = *r {
                return Arc::make_mut(map.entry(key).or_insert(Arc::new(JsonPart::Null)));
            }
            panic!();
        }

        fn _get_array_mut(r: &mut JsonPart, key: usize) -> &mut JsonPart {
            if let JsonPart::Null = *r {
                *r = JsonPart::Array(Vec::new());
            }
            if let JsonPart::Array(ref mut arr) = *r {
                while key >= arr.len() {
                    arr.push(Arc::new(JsonPart::Null));
                }
                return Arc::make_mut(&mut arr[key]);
            }
            panic!();
        }

        return path.split('/').fold(Arc::make_mut(&mut self.0), |r, part| {
            if part.starts_with('#') {
                return _get_array_mut(r, part[1..].parse().unwrap());
            }
            return _get_hash_mut(r, Arc::from(part));
        });
    }
}

impl FromStr for Record {
    type Err = serde_json::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn convert_part(p: &serde_json::value::Value) -> JsonPart {
            return match p {
                serde_json::value::Value::Null => JsonPart::Null,
                serde_json::value::Value::Bool(b) => JsonPart::Bool(*b),
                serde_json::value::Value::Number(n) => JsonPart::Number(n.clone()),
                serde_json::value::Value::String(s) => JsonPart::String(Arc::from(s.clone())),
                serde_json::value::Value::Array(arr) => JsonPart::Array(arr.iter().map(|v| Arc::new(convert_part(v))).collect()),
                serde_json::value::Value::Object(map) => JsonPart::Hash(map.iter().map(|(k, v)| (Arc::from(k.clone()), Arc::new(convert_part(v)))).collect()),
            }
        }

        return serde_json::from_str(s).map(|j| Record(Arc::new(convert_part(&j))));
    }
}

impl ToString for Record {
    fn to_string(&self) -> String {
        fn _to_string_aux(p: &JsonPart, acc: &mut String) {
            match p {
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
        _to_string_aux(&self.0, &mut ret);
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
        assert_eq!(r.get_path(Arc::from("x")).to_string(), "[{\"y\":\"z\"}]");
        assert_eq!(r.get_path(Arc::from("y")).to_string(), "null");
        assert_eq!(r.get_path(Arc::from("y/z")).to_string(), "null");
        assert_eq!(r.get_path(Arc::from("x/#0")).to_string(), "{\"y\":\"z\"}");
        assert_eq!(r.get_path(Arc::from("x/#1")).to_string(), "null");
        assert_eq!(r.get_path(Arc::from("x/#0/y")).to_string(), "\"z\"");
    }

    #[test]
    fn test_set_path() {
        let mut r = Record::from_str("{\"x\":[{\"y\":\"z\"}]}").unwrap();
        let r2 = r.clone();
        *r.get_path_mut(Arc::from("x/#0/y")) = JsonPart::String(Arc::from("w"));
        assert_eq!(r.to_string(), "{\"x\":[{\"y\":\"w\"}]}");
        assert_eq!(r2.to_string(), "{\"x\":[{\"y\":\"z\"}]}");
        *r.get_path_mut(Arc::from("a/#2/b")) = JsonPart::String(Arc::from("c"));
        assert_eq!(r.to_string(), "{\"a\":[null,null,{\"b\":\"c\"}],\"x\":[{\"y\":\"w\"}]}");
    }
}
