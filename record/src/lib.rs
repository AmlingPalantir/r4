use std::collections::BTreeMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::sync::Arc;
use std::vec::Vec;

#[cfg(test)]
mod tests;

#[derive(Clone)]
struct F64Wrapper(f64);

impl F64Wrapper {
    fn to_json_string(&self) -> String {
        return serde_json::to_string(&serde_json::Number::from_f64(self.0)).unwrap();
    }
}

// Ouch, if it comes down to Hash/Eq for F64Wrapper you've really asked for it,
// but we do the least bad, least insane thing we can...

impl Eq for F64Wrapper {
}

impl PartialEq for F64Wrapper {
    fn eq(&self, other: &F64Wrapper) -> bool {
        return self.to_json_string() == other.to_json_string();
    }
}

impl Hash for F64Wrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_json_string().hash(state);
    }
}

#[derive(Clone)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
enum JsonPart {
    Null,
    Bool(bool),
    NumberI64(i64),
    NumberF64(F64Wrapper),
    String(Arc<str>),
    Array(Vec<Record>),
    Hash(BTreeMap<Arc<str>, Record>),
}

impl JsonPart {
    fn from_serde_number(n: &serde_json::Number) -> JsonPart {
        if let Some(n) = n.as_i64() {
            return JsonPart::NumberI64(n);
        }
        if let Some(n) = n.as_f64() {
            return JsonPart::NumberF64(F64Wrapper(n));
        }
        panic!("Unhandled JSON number type: {}", n);
    }
}

#[derive(Clone)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub struct Record(Arc<JsonPart>);

pub trait FromPrimitive<P> {
    fn from_primitive(p: P) -> Self;
}

impl FromPrimitive<i64> for Record {
    fn from_primitive(n: i64) -> Self {
        return Record(Arc::new(JsonPart::NumberI64(n)));
    }
}

impl Record {
    pub fn null() -> Self {
        return Record(Arc::new(JsonPart::Null));
    }

    pub fn empty_hash() -> Self {
        return Record(Arc::new(JsonPart::Hash(BTreeMap::new())));
    }

    pub fn from_str<S: Deref<Target = str>>(s: S) -> Self {
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
        panic!("Record::get_hash() on non-hash");
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
        panic!("Record::get_array() on non-array");
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
            panic!("Record::_get_hash_mut() on non-hash");
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
            panic!("Record::_get_array_mut() on non-array");
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

    pub fn parse(s: &str) -> Self {
        fn convert_part(p: &serde_json::value::Value) -> Record {
            return Record(Arc::new(match p {
                serde_json::value::Value::Null => JsonPart::Null,
                serde_json::value::Value::Bool(b) => JsonPart::Bool(*b),
                serde_json::value::Value::Number(n) => JsonPart::from_serde_number(n),
                serde_json::value::Value::String(s) => JsonPart::String(Arc::from(s.clone())),
                serde_json::value::Value::Array(arr) => JsonPart::Array(arr.iter().map(|v| convert_part(v)).collect()),
                serde_json::value::Value::Object(map) => JsonPart::Hash(map.iter().map(|(k, v)| (Arc::from(k.clone()), convert_part(v))).collect()),
            }));
        }

        return convert_part(&serde_json::from_str(s).unwrap());
    }

    pub fn deparse(&self) -> String {
        fn _to_string_aux(p: &Record, acc: &mut String) {
            match &*p.0 {
                JsonPart::Null => {
                    acc.push_str("null");
                }
                JsonPart::Bool(b) => {
                    acc.push_str(if *b { "true" } else { "false" });
                }
                JsonPart::NumberI64(n) => {
                    acc.push_str(&serde_json::to_string(&serde_json::Number::from(*n)).unwrap());
                }
                JsonPart::NumberF64(n) => {
                    acc.push_str(&n.to_json_string());
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

    pub fn expect_string(&self) -> Arc<str> {
        return match *self.0 {
            JsonPart::String(ref s) => s.clone(),
            _ => panic!(),
        };
    }
}
