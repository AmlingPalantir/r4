#[macro_use]
extern crate lalrpop_util;
extern crate misc;
extern crate record;

mod ast;
lalrpop_mod!(pub parse);

use ast::BinaryOp;
use ast::Expr;
use ast::UnaryOp;
use record::Record;
use std::sync::Arc;

struct State {
    r: Record,
}

impl State {
    fn eval_binary_number_op<RI, FI: FnOnce(i64, i64) -> RI, RF, FF: FnOnce(f64, f64) -> RF>(&mut self, e1: &Expr, e2: &Expr, fi: FI, ff: FF) -> Record where Record: From<RI> + From<RF> {
        let r1 = self.eval(e1);
        let r2 = self.eval(e2);

        if let Some(i1) = r1.maybe_i64() {
            if let Some(i2) = r2.maybe_i64() {
                return Record::from(fi(i1, i2));
            }
        }
        if let Some(f1) = r1.maybe_num() {
            if let Some(f2) = r2.maybe_num() {
                return Record::from(ff(f1, f2));
            }
        }

        panic!();
    }

    fn eval_binary_string_op<R, F: FnOnce(Arc<str>, Arc<str>) -> R>(&mut self, e1: &Expr, e2: &Expr, f: F) -> Record where Record: From<R> {
        let s1 = self.eval(e1).coerce_string();
        let s2 = self.eval(e2).coerce_string();

        return Record::from(f(s1, s2));
    }

    fn eval(&mut self, e: &Expr) -> Record {
        match e {
            Expr::Ternary(e1, e2, e3) => {
                if self.eval(e1).coerce_bool() {
                    return self.eval(e2);
                }
                return self.eval(e3);
            }

            Expr::Binary(e1, BinaryOp::LogOr(), e2) => {
                if self.eval(e1).coerce_bool() {
                    return Record::from(true);
                }
                return self.eval(e2);
            }
            Expr::Binary(e1, BinaryOp::LogAnd(), e2) => {
                if !self.eval(e1).coerce_bool() {
                    return Record::from(false);
                }
                return self.eval(e2);
            }
            Expr::Unary(UnaryOp::LogNeg(), e1) => {
                return Record::from(!self.eval(e1).coerce_bool());
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
                let r1 = self.eval(e1);
                if let Some(i) = r1.maybe_i64() {
                    return Record::from(-i);
                }
                return Record::from(-r1.coerce_f64());
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

            Expr::Literal(r) => {
                return r.clone();
            }
            Expr::ArrayLiteral(es) => {
                return Record::from_vec(es.iter().map(|e| self.eval(e)).collect());
            }
            Expr::HashLiteral(es) => {
                return Record::from_hash(es.iter().map(|(k, v)| (k.clone(), self.eval(v))).collect());
            }
        }
    }
}

pub fn load(code: &str) -> Box<Fn(Record) -> Record> {
    let es = parse::StmtParser::new().parse(code).unwrap();
    return Box::new(move |r| {
        let mut st = State {
            r: r,
        };
        for e in es.iter() {
            st.eval(e);
        }
        return st.r;
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        //assert_eq!(super::parse::Expr1Parser::new().parse("1 + (1 + 2) * 3").unwrap(), 10);
    }
}
