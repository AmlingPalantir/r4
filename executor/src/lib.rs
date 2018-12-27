#[macro_use]
extern crate lalrpop_util;
extern crate misc;
extern crate record;

mod ast;
lalrpop_mod!(pub parse);

use ast::BinaryOp;
use ast::Expr;
use ast::UnaryOp;
use misc::Either;
use record::JsonPart;
use record::JsonPrimitive;
use record::Record;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
struct ValueFe(Arc<Mutex<Either<Record, ValueBe>>>);

impl ValueFe {
    fn from_record(r: Record) -> ValueFe {
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

    fn with_be<R, F: FnOnce(&mut ValueBe) -> R>(&mut self, f: F) -> R {
        let mut be = self.0.lock().unwrap();
        return f((*be).convert_r_mut(|r| ValueBe::convert(r)));
    }

    fn get_path(&mut self, path: &str) -> ValueFe {
        return self.with_be(|be| be.get_path(path));
    }

    fn set_path(&mut self, path: &str, v: ValueFe) {
        return self.with_be(|be| be.set_path(path, v));
    }

    fn del_path(&mut self, path: &str) -> ValueFe {
        return self.with_be(|be| be.del_path(path));
    }

    fn coerce_bool(&self) -> bool {
        return self.to_record().coerce_bool();
    }

    fn coerce_string(&self) -> Arc<str> {
        return self.to_record().coerce_string();
    }
}

impl<T> From<T> for ValueFe where JsonPrimitive: From<T> {
    fn from(t: T) -> Self {
        return ValueFe(Arc::new(Mutex::new(Either::Right(ValueBe::Primitive(JsonPrimitive::from(t))))));
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
                return ValueBe::Array(arr.iter().map(|e| ValueFe::from_record(e.clone())).collect());
            }
            JsonPart::Hash(ref hash) => {
                return ValueBe::Hash(hash.iter().map(|(k, v)| (k.clone(), ValueFe::from_record(v.clone()))).collect());
            }
        }
    }

    fn to_record(&self) -> Record {
        match self {
            ValueBe::Primitive(p) => {
                return Record::from(p.clone());
            }
            ValueBe::Array(arr) => {
                return Record::from_vec(arr.iter().map(|e| e.to_record()).collect());
            }
            ValueBe::Hash(hash) => {
                return Record::from_hash(hash.iter().map(|(k, v)| (k.clone(), v.to_record())).collect());
            }
        }
    }

    fn get_path(&mut self, _path: &str) -> ValueFe {
        unimplemented!();
    }

    fn set_path(&mut self, _path: &str, _v: ValueFe) {
        unimplemented!();
    }

    fn del_path(&mut self, _path: &str) -> ValueFe {
        unimplemented!();
    }
}

struct State {
    r: ValueFe,
}

impl State {
    fn eval_binary_number_op<RI, FI: FnOnce(i64, i64) -> RI, RF, FF: FnOnce(f64, f64) -> RF>(&mut self, e1: &Expr, e2: &Expr, fi: FI, ff: FF) -> ValueFe where ValueFe: From<RI> + From<RF> {
        let r1 = self.eval(e1).to_record();
        let r2 = self.eval(e2).to_record();

        if let Some(i1) = r1.maybe_i64() {
            if let Some(i2) = r2.maybe_i64() {
                return ValueFe::from(fi(i1, i2));
            }
        }
        if let Some(f1) = r1.maybe_num() {
            if let Some(f2) = r2.maybe_num() {
                return ValueFe::from(ff(f1, f2));
            }
        }

        panic!();
    }

    fn eval_binary_string_op<R, F: FnOnce(Arc<str>, Arc<str>) -> R>(&mut self, e1: &Expr, e2: &Expr, f: F) -> ValueFe where ValueFe: From<R> {
        let s1 = self.eval(e1).coerce_string();
        let s2 = self.eval(e2).coerce_string();

        return ValueFe::from(f(s1, s2));
    }

    fn eval(&mut self, e: &Expr) -> ValueFe {
        match e {
            Expr::Ternary(e1, e2, e3) => {
                if self.eval(e1).coerce_bool() {
                    return self.eval(e2);
                }
                return self.eval(e3);
            }

            Expr::Binary(e1, BinaryOp::LogOr(), e2) => {
                if self.eval(e1).coerce_bool() {
                    return ValueFe::from(true);
                }
                return self.eval(e2);
            }
            Expr::Binary(e1, BinaryOp::LogAnd(), e2) => {
                if !self.eval(e1).coerce_bool() {
                    return ValueFe::from(false);
                }
                return self.eval(e2);
            }
            Expr::Unary(UnaryOp::LogNeg(), e1) => {
                return ValueFe::from(!self.eval(e1).coerce_bool());
            }

            Expr::Binary(e1, BinaryOp::NumLt(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 < i2, |f1, f2| f1 < f2);
            }
            Expr::Binary(e1, BinaryOp::NumLte(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 <= i2, |f1, f2| f1 <= f2);
            }
            Expr::Binary(e1, BinaryOp::NumGt(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 > i2, |f1, f2| f1 > f2);
            }
            Expr::Binary(e1, BinaryOp::NumGte(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 >= i2, |f1, f2| f1 >= f2);
            }
            Expr::Binary(e1, BinaryOp::NumEq(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 == i2, |f1, f2| f1 == f2);
            }
            Expr::Binary(e1, BinaryOp::NumNe(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 != i2, |f1, f2| f1 != f2);
            }

            Expr::Binary(e1, BinaryOp::Lt(), e2) => {
                return self.eval_binary_string_op(e1, e2, |s1, s2| s1 < s2);
            }
            Expr::Binary(e1, BinaryOp::Lte(), e2) => {
                return self.eval_binary_string_op(e1, e2, |s1, s2| s1 <= s2);
            }
            Expr::Binary(e1, BinaryOp::Gt(), e2) => {
                return self.eval_binary_string_op(e1, e2, |s1, s2| s1 > s2);
            }
            Expr::Binary(e1, BinaryOp::Gte(), e2) => {
                return self.eval_binary_string_op(e1, e2, |s1, s2| s1 >= s2);
            }
            Expr::Binary(e1, BinaryOp::Eq(), e2) => {
                return self.eval_binary_string_op(e1, e2, |s1, s2| s1 == s2);
            }
            Expr::Binary(e1, BinaryOp::Ne(), e2) => {
                return self.eval_binary_string_op(e1, e2, |s1, s2| s1 != s2);
            }

            Expr::Binary(e1, BinaryOp::Add(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 + i2, |f1, f2| f1 + f2);
            }
            Expr::Binary(e1, BinaryOp::Sub(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 - i2, |f1, f2| f1 - f2);
            }
            Expr::Binary(e1, BinaryOp::Mul(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 * i2, |f1, f2| f1 * f2);
            }
            Expr::Binary(e1, BinaryOp::Div(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 / i2, |f1, f2| f1 / f2);
            }
            Expr::Binary(e1, BinaryOp::Mod(), e2) => {
                return self.eval_binary_number_op(e1, e2, |i1, i2| i1 % i2, |f1, f2| f1 % f2);
            }

            Expr::Unary(UnaryOp::NumNeg(), e1) => {
                let r1 = self.eval(e1).to_record();
                if let Some(i) = r1.maybe_i64() {
                    return ValueFe::from(-i);
                }
                return ValueFe::from(-r1.coerce_f64());
            }

            Expr::Binary(e1, BinaryOp::Cat(), e2) => {
                return self.eval_binary_string_op(e1, e2, |s1, s2| {
                    let mut s = "".to_string();
                    s.push_str(&s1);
                    s.push_str(&s2);
                    return Arc::from(s);
                });
            }

            Expr::RecordRead(s) => {
                return self.r.get_path(s);
            }
            Expr::RecordWrite(s, e) => {
                let v = self.eval(e);
                self.r.set_path(s, v.clone());
                return v;
            }
            Expr::RecordDelete(s) => {
                return self.r.del_path(s);
            }

            _ => {
                unimplemented!();
            }
        }
    }
}

pub fn load(code: &str) -> Box<Fn(Record) -> Record> {
    let es = parse::StmtParser::new().parse(code).unwrap();
    return Box::new(move |r| {
        let mut st = State {
            r: ValueFe::from_record(r),
        };
        for e in es.iter() {
            st.eval(e);
        }
        return st.r.to_record();
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        //assert_eq!(super::parse::Expr1Parser::new().parse("1 + (1 + 2) * 3").unwrap(), 10);
    }
}
