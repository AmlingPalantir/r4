#[cfg(test)]
mod tests;

use record::MRecord;
use record::Record;
use record::RecordNode;
use record::RecordTrait;
use rlua::Lua;
use rlua::MetaMethod;
use rlua::UserData;
use rlua::UserDataMethods;
use rlua::Value;
use std::sync::Arc;

#[derive(Clone)]
struct MRecordHolder(MRecord);

impl UserData for MRecordHolder {
    fn add_methods<'lua, M: UserDataMethods<'lua, MRecordHolder>>(m: &mut M) {
        m.add_meta_method_mut(MetaMethod::Index, |lua, r, k: Value| {
            return r.0.visit_converted(
                |rn| {
                    match rn {
                        RecordNode::Primitive(_p) => {
                            panic!();
                        }
                        RecordNode::Array(arr) => {
                            let k = lua.coerce_integer(k).unwrap() as usize;
                            return Result::Ok(MRecordHolder(arr[k - 1].clone()));
                        }
                        RecordNode::Hash(hash) => {
                            let k: Arc<str> = Arc::from(lua.coerce_string(k).unwrap().to_str().unwrap());
                            return Result::Ok(MRecordHolder(hash.get(&k).unwrap().clone()));
                        }
                    }
                }
            );
        });
        m.add_meta_method_mut(MetaMethod::NewIndex, |lua, r, (k, v): (Value, Value)| {
            let v: MRecord = from_lua(lua, v);
            return r.0.visit_converted(
                |rn| {
                    match rn {
                        RecordNode::Primitive(_p) => {
                            panic!();
                        }
                        RecordNode::Array(arr) => {
                            let k = lua.coerce_integer(k).unwrap() as usize;
                            arr[k - 1] = v;
                        }
                        RecordNode::Hash(hash) => {
                            let k: Arc<str> = Arc::from(lua.coerce_string(k).unwrap().to_str().unwrap());
                            hash.insert(k, v);
                        }
                    }
                    return Result::Ok(());
                }
            );
        });
    }
}

fn from_lua(lua: &Lua, v: Value) -> MRecord {
    match v {
        Value::Nil => {
            return MRecord::null();
        }
        Value::Boolean(b) => {
            return MRecord::from(b);
        }
        Value::Integer(n) => {
            return MRecord::from(n);
        }
        Value::Number(n) => {
            return MRecord::from(n as f64);
        }
        Value::String(s) => {
            return MRecord::from(s.to_str().unwrap());
        }
        Value::Table(t) => {
            return MRecord::from_hash(t.pairs::<Value, Value>().map(|p| {
                let (k, v) = p.unwrap();
                let k = Arc::from(lua.coerce_string(k).unwrap().to_str().unwrap());
                return (k, from_lua(lua, v));
            }).collect());
        }
        Value::UserData(ud) => {
            if let Result::Ok(r) = ud.borrow::<MRecordHolder>() {
                return r.0.clone();
            }
            panic!();
        }
        _ => {
            panic!();
        }
    }
}

pub fn stream(code: &str, ret: bool) -> Box<FnMut(Record) -> Record> {
    let code = code.to_string();
    let lua = Lua::new();
    return Box::new(move |r| {
        lua.globals().set("r", MRecordHolder(MRecord::wrap(r))).unwrap();
        let r: Value;
        if ret {
            r = lua.eval(&code, None).unwrap();
        }
        else {
            let _: () = lua.eval(&code, None).unwrap();
            r = lua.globals().get("r").unwrap();
        }
        return from_lua(&lua, r).to_record();
    });
}
