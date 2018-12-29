#[macro_use]
extern crate lalrpop_util;
extern crate misc;
extern crate record;

mod ast;
lalrpop_mod!(pub parse);

#[cfg(test)]
mod tests;

use ast::BinaryOp;
use ast::Expr;
use ast::UnaryOp;
use misc::Either;
use record::MRecord;
use record::Record;
use record::RecordTrait;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
struct State {
    vars: HashMap<Arc<str>, MRecord>,
}

impl State {
    fn eval_binary_number_op<RI, FI: FnOnce(i64, i64) -> RI, RF, FF: FnOnce(f64, f64) -> RF>(&mut self, e1: &Expr, e2: &Expr, fi: FI, ff: FF) -> MRecord where MRecord: From<RI> + From<RF> {
        let n1 = self.eval(e1).coerce_num();
        let n2 = self.eval(e2).coerce_num();

        if let Either::Left(i1) = n1 {
            if let Either::Left(i2) = n2 {
                return MRecord::from(fi(i1, i2));
            }
        }

        let f1 = n1.map_left(|i| i as f64).join();
        let f2 = n2.map_left(|i| i as f64).join();

        return MRecord::from(ff(f1, f2));
    }

    fn eval_binary_string_op<R, F: FnOnce(Arc<str>, Arc<str>) -> R>(&mut self, e1: &Expr, e2: &Expr, f: F) -> MRecord where MRecord: From<R> {
        let s1 = self.eval(e1).coerce_string();
        let s2 = self.eval(e2).coerce_string();

        return MRecord::from(f(s1, s2));
    }

    fn eval(&mut self, e: &Expr) -> MRecord {
        match e {
            Expr::Statement(es) => {
                let mut ret = MRecord::null();
                for e in es {
                    ret = self.eval(e);
                }
                return ret;
            }
            Expr::Ternary(e1, e2, e3) => {
                if self.eval(e1).coerce_bool() {
                    return self.eval(e2);
                }
                return self.eval(e3);
            }

            Expr::Binary(e1, BinaryOp::LogOr(), e2) => {
                let v1 = self.eval(e1);
                if v1.coerce_bool() {
                    return v1;
                }
                return self.eval(e2);
            }
            Expr::Binary(e1, BinaryOp::LogAnd(), e2) => {
                let v1 = self.eval(e1);
                if !v1.coerce_bool() {
                    return v1;
                }
                return self.eval(e2);
            }
            Expr::Unary(UnaryOp::LogNeg(), e1) => {
                return MRecord::from(!self.eval(e1).coerce_bool());
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

            Expr::Unary(UnaryOp::NumNeg(), e) => {
                let n = self.eval(e).coerce_num();
                let n = n.map_left(|i| MRecord::from(-i));
                let n = n.map_right(|f| MRecord::from(-f));
                return n.join();
            }

            Expr::Binary(e1, BinaryOp::Cat(), e2) => {
                return self.eval_binary_string_op(e1, e2, |s1, s2| {
                    let mut s = "".to_string();
                    s.push_str(&s1);
                    s.push_str(&s2);
                    return Arc::from(s);
                });
            }

            Expr::RecordRead(e, s) => {
                return self.eval(e).get_path_obj(s);
            }
            Expr::RecordReadFill(e, s) => {
                return self.eval(e).get_path_obj_fill(s);
            }
            Expr::RecordWrite(e, s, e2) => {
                let mut r = self.eval(e);
                let v = self.eval(e2);
                r.set_path_obj(s, v.clone());
                return v;
            }
            Expr::RecordDelete(e, s) => {
                return self.eval(e).del_path_obj(s);
            }

            Expr::Literal(r) => {
                return MRecord::wrap(r.clone());
            }
            Expr::ArrayLiteral(es) => {
                return MRecord::from_vec(es.iter().map(|e| self.eval(e)).collect());
            }
            Expr::HashLiteral(es) => {
                return MRecord::from_hash(es.iter().map(|(k, v)| (k.clone(), self.eval(v))).collect());
            }

            Expr::WriteVar(s, e) => {
                let v = self.eval(e);
                self.vars.insert(s.clone(), v.clone());
                return v;
            }
            Expr::ReadVar(s) => {
                return self.vars.entry(s.clone()).or_insert_with(MRecord::null).clone();
            }
        }
    }
}

#[derive(Clone)]
pub struct Code(Arc<Box<Expr>>);

impl Code {
    pub fn parse(code: &str) -> Self {
        return Code(Arc::new(parse::StatementParser::new().parse(code).unwrap()));
    }

    pub fn stream(&self) -> Box<FnMut(Record) -> (Record, Record)> {
        let e = self.0.clone();
        let mut st = State::default();
        return Box::new(move |r| {
            st.vars.insert(Arc::from("r"), MRecord::wrap(r));
            let ret = st.eval(&e);
            return (ret.to_record(), st.vars.get("r").unwrap().clone().to_record());
        });
    }
}
