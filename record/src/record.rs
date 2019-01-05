use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec::Vec;
use super::JsonPrimitive;
use super::Path;
use super::RecordNode;
use super::RecordTrait;

#[derive(Clone)]
#[derive(Debug)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub struct Record(pub(crate) Arc<RecordNode<Record>>);

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
        let mut path = Path::new(path).0.into_iter();
        let first = path.next().expect("Delete of empty path?");
        let (r, part) = path.fold(
            (self, first),
            |(r, prev), part| {
                let r = Arc::make_mut(&mut r.0).get_rstep_fill(&prev);
                return (r, part);
            }
        );
        return Arc::make_mut(&mut r.0).del_rpart(&part);
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
                    acc.push_str(&serde_json::to_string(s as &str).unwrap());
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
                        acc.push_str(&serde_json::to_string(k as &str).unwrap());
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

    pub fn expect_array(&self) -> &Vec<Record> {
        return match *self.0 {
            RecordNode::Array(ref arr) => arr,
            _ => panic!("expect_array() on non-array"),
        };
    }

    pub fn expect_hash(&self) -> &BTreeMap<Arc<str>, Record> {
        return match *self.0 {
            RecordNode::Hash(ref hash) => hash,
            _ => panic!("expect_hash() on non-hash"),
        };
    }

    pub fn pretty_string(&self) -> String {
        return match *self.0 {
            RecordNode::Primitive(JsonPrimitive::String(ref s)) => s.to_string(),
            _ => self.deparse(),
        };
    }
}
