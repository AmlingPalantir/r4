#[derive(Clone)]
struct Record(Arc<JsonPart>);

use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Clone)]
enum JsonPart {
    Primitive(JsonPrimitive),
    // TODO: less crummy Vec (better splice?)
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

    fn get_path(&self, path: Arc<str>) -> Record {
        let mut ret = self.clone();
        let null = Record(Arc::new(JsonPart::Primitive(JsonPrimitive::Null)));

        for part in path.split('/') {
            let next;
            next = ret.get_hash(Arc::from(part));
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

        fn _get_path_mut<'a, I: Iterator<Item = &'a str>>(r: &mut JsonPart, mut parts: I) -> &mut JsonPart {
            match parts.next() {
                Some(part) => {
                    return _get_path_mut(_get_hash_mut(r, Arc::from(part)), parts);
                }
                None => {
                    return r;
                }
            }
        }

        return _get_path_mut(Arc::make_mut(&mut self.0), path.split('/'));
    }
}
