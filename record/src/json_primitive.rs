use std::sync::Arc;
use super::F64HashDishonorProxy;

#[derive(Clone)]
#[derive(Debug)]
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
    pub fn from_serde_number(n: &serde_json::Number) -> JsonPrimitive {
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
