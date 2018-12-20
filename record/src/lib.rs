#[derive(Clone)]
struct Record(Arc<JsonPart>);

use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Clone)]
enum JsonPart {
    Primitive(JsonPrimitive),
    // TODO: less crummy Vec (better splice?)
    Array(Vec<Arc<JsonPart>>),
    Hash(BTreeMap<Arc<str>, Arc<JsonPart>>),
}

#[derive(Clone)]
enum JsonPrimitive {
    // TODO: real versions, presumably from underlying json library?
    Null,
    String(Arc<str>),
    Number(f32),
}

impl Record {
    fn get_hash(&self, key: Arc<str>) -> Option<Record> {
        if let JsonPart::Hash(ref map) = *self.0 {
            return match map.get(&key) {
                Some(arc) => Some(Record(arc.clone())),
                None => None,
            };
        }
        panic!();
    }

    fn get_array(&self, key: usize) -> Option<Record> {
        if let JsonPart::Array(ref arr) = *self.0 {
            return match arr.get(key) {
                Some(arc) => Some(Record(arc.clone())),
                None => None,
            };
        }
        panic!();
    }

    fn get_path(&self, path: Arc<str>) -> Record {
        let mut ret = self.clone();
        let null = Record(Arc::new(JsonPart::Primitive(JsonPrimitive::Null)));

        for part in path.split('/') {
            let next;
            if part.starts_with('#') {
                next = ret.get_array(part[1..].parse().unwrap())
            }
            else {
                next = ret.get_hash(Arc::from(part));
            }
            match next {
                Some(next) => {
                    ret = next;
                }
                None => {
                    ret = null.clone();
                }
            }
        }

        return ret;
    }

    fn get_path_mut(&mut self, path: Arc<str>) -> &mut JsonPart {
        fn _get_hash_mut(r: &mut JsonPart, key: Arc<str>) -> &mut JsonPart {
            if let JsonPart::Primitive(JsonPrimitive::Null) = *r {
                *r = JsonPart::Hash(BTreeMap::new());
            }
            if let JsonPart::Hash(ref mut map) = *r {
                return Arc::make_mut(map.entry(key).or_insert(Arc::new(JsonPart::Primitive(JsonPrimitive::Null))));
            }
            panic!();
        }

        fn _get_array_mut(r: &mut JsonPart, key: usize) -> &mut JsonPart {
            if let JsonPart::Primitive(JsonPrimitive::Null) = *r {
                *r = JsonPart::Array(Vec::new());
            }
            if let JsonPart::Array(ref mut arr) = *r {
                if key < 0 {
                    panic!();
                }
                while key >= arr.len() {
                    arr.push(Arc::new(JsonPart::Primitive(JsonPrimitive::Null)));
                }
                return Arc::make_mut(&mut arr[key]);
            }
            panic!();
        }

        return path.split('/').fold(Arc::make_mut(&mut self.0), |r, part| {
            if part.starts_with('#') {
                return _get_array_mut(r, part[1..].parse().unwrap());
            }
            else {
                return _get_hash_mut(r, Arc::from(part));
            }
        });
    }
}
