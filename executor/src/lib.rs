extern crate record;

use record::JsonPart;
use record::JsonPrimitive;
use record::Record;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
struct HolderFe(Arc<Mutex<HolderBe>>);

impl HolderFe {
    fn new(r: Record) -> HolderFe {
        return HolderFe(Arc::new(Mutex::new(HolderBe::Unconverted(r))));
    }

    fn from_value(v: Value) -> HolderFe {
        match v {
            Value::Nil => {
                return HolderFe::new(Record::null());
            }
            Value::Boolean(b) => {
                return HolderFe::new(Record::from_bool(b));
            }
            Value::Integer(n) => {
                return HolderFe::new(Record::from_i64(n as i64));
            }
            Value::Number(n) => {
                return HolderFe::new(Record::from_f64(n as f64));
            }
            Value::String(s) => {
                return HolderFe::new(Record::from_str(s.to_str().unwrap()));
            }
            Value::Table(t) => {
                let c = HolderBeConverted::Hash(t.pairs::<Value, Value>().map(|p| {
                    let (k, v) = p.unwrap();
                    let k = HolderFe::from_value(k).to_record().coerce_string();
                    return (k, HolderFe::from_value(v));
                }).collect());
                return HolderFe(Arc::new(Mutex::new(HolderBe::Converted(c))));
            }
            Value::UserData(ud) => {
                if let Result::Ok(fe) = ud.borrow::<HolderFe>() {
                    return fe.clone();
                }
                panic!();
            }
            _ => {
                panic!();
            }
        }
    }

    fn to_record(&self) -> Record {
        let be = self.0.lock().unwrap();
        return be.to_record();
    }
}

enum HolderBe {
    Unconverted(Record),
    Converted(HolderBeConverted),
}

impl HolderBe {
    fn convert(&mut self) -> &mut HolderBeConverted {
        // Arggh borrow checked, you really make this kind of thing a mess...
        let c;
        match self {
            HolderBe::Converted(c) => {
                return c;
            }
            HolderBe::Unconverted(r) => {
                c = HolderBeConverted::convert(r);
            }
        }
        *self = HolderBe::Converted(c);
        match self {
            HolderBe::Converted(c) => {
                return c;
            }
            HolderBe::Unconverted(_r) => {
                panic!();
            }
        }
    }

    fn to_record(&self) -> Record {
        match self {
            HolderBe::Unconverted(r) => {
                return r.clone();
            }
            HolderBe::Converted(c) => {
                return c.to_record();
            }
        }
    }
}

enum HolderBeConverted {
    Primitive(JsonPrimitive),
    Array(Vec<HolderFe>),
    Hash(BTreeMap<Arc<str>, HolderFe>),
}

impl HolderBeConverted {
    fn convert(r: &Record) -> HolderBeConverted {
        match *r.0 {
            JsonPart::Primitive(ref p) => {
                return HolderBeConverted::Primitive(p.clone());
            }
            JsonPart::Array(ref arr) => {
                return HolderBeConverted::Array(arr.iter().map(|e| HolderFe::new(e.clone())).collect());
            }
            JsonPart::Hash(ref hash) => {
                return HolderBeConverted::Hash(hash.iter().map(|(k, v)| (k.clone(), HolderFe::new(v.clone()))).collect());
            }
        }
    }

    fn to_record(&self) -> Record {
        match self {
            HolderBeConverted::Primitive(p) => {
                return Record::from_json_primitive(p.clone());
            }
            HolderBeConverted::Array(arr) => {
                return Record::from_vec(arr.iter().map(|e| e.to_record()).collect());
            }
            HolderBeConverted::Hash(hash) => {
                return Record::from_hash(hash.iter().map(|(k, v)| (k.clone(), v.to_record())).collect());
            }
        }
    }
}

impl UserData for HolderFe {
    fn add_methods<'lua, M: UserDataMethods<'lua, HolderFe>>(m: &mut M) {
        m.add_meta_method_mut(MetaMethod::Index, |lua, fe, k: Value| {
            let mut be = fe.0.lock().unwrap();
            match be.convert() {
                HolderBeConverted::Primitive(_p) => {
                    panic!();
                }
                HolderBeConverted::Array(arr) => {
                    let k = lua.coerce_integer(k).unwrap() as usize;
                    return Result::Ok(arr[k - 1].clone());
                }
                HolderBeConverted::Hash(hash) => {
                    let k: Arc<str> = Arc::from(lua.coerce_string(k).unwrap().to_str().unwrap());
                    return Result::Ok(hash.get(&k).unwrap().clone());
                }
            }
        });
        m.add_meta_method_mut(MetaMethod::NewIndex, |lua, fe, (k, v): (Value, Value)| {
            let v = HolderFe::from_value(v);
            let mut be = fe.0.lock().unwrap();
            match be.convert() {
                HolderBeConverted::Primitive(_p) => {
                    panic!();
                }
                HolderBeConverted::Array(arr) => {
                    let k = lua.coerce_integer(k).unwrap() as usize;
                    arr[k - 1] = v;
                }
                HolderBeConverted::Hash(hash) => {
                    let k: Arc<str> = Arc::from(lua.coerce_string(k).unwrap().to_str().unwrap());
                    hash.insert(k, v);
                }
            }
            return Result::Ok(());
        });
    }
}

pub fn load(code: &str) -> Box<Fn(Record) -> Record> {
    let code = code.to_string();
    return Box::new(move |r| {
        let lua = Lua::new();
        lua.globals().set("r", HolderFe::new(r)).unwrap();
        let _: () = lua.eval(&code, None).unwrap();
        let r: HolderFe = lua.globals().get("r").unwrap();
        return r.to_record();
    });
}
