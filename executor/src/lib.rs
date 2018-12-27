#[macro_use]
extern crate lalrpop_util;
extern crate misc;
extern crate record;

mod ast;
lalrpop_mod!(pub parse);

use record::JsonPart;
use record::JsonPrimitive;
use record::Record;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use misc::Either;

#[derive(Clone)]
struct ValueFe(Arc<Mutex<Either<Record, ValueBe>>>);

impl ValueFe {
    fn new(r: Record) -> ValueFe {
        return ValueFe(Arc::new(Mutex::new(Either::Left(r))));
    }

    fn to_record(&self) -> Record {
        let be = self.0.lock().unwrap();
        match *be {
            Either::Left(ref r) => {
                return r.clone();
            }
            Either::Right(ref be) => {
                return be.to_record();
            }
        }
    }
}

enum ValueBe {
    Primitive(JsonPrimitive),
    Array(Vec<ValueFe>),
    Hash(BTreeMap<Arc<str>, ValueFe>),
}

impl ValueBe {
    fn convert(r: &Record) -> ValueBe {
        match *r.0 {
            JsonPart::Primitive(ref p) => {
                return ValueBe::Primitive(p.clone());
            }
            JsonPart::Array(ref arr) => {
                return ValueBe::Array(arr.iter().map(|e| ValueFe::new(e.clone())).collect());
            }
            JsonPart::Hash(ref hash) => {
                return ValueBe::Hash(hash.iter().map(|(k, v)| (k.clone(), ValueFe::new(v.clone()))).collect());
            }
        }
    }

    fn to_record(&self) -> Record {
        match self {
            ValueBe::Primitive(p) => {
                return Record::from_json_primitive(p.clone());
            }
            ValueBe::Array(arr) => {
                return Record::from_vec(arr.iter().map(|e| e.to_record()).collect());
            }
            ValueBe::Hash(hash) => {
                return Record::from_hash(hash.iter().map(|(k, v)| (k.clone(), v.to_record())).collect());
            }
        }
    }
}

pub fn load(_code: &str) -> Box<Fn(Record) -> Record> {
    return Box::new(move |r| {
        return r;
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        //assert_eq!(super::parse::Expr1Parser::new().parse("1 + (1 + 2) * 3").unwrap(), 10);
    }
}
