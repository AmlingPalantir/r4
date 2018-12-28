use record::Record;
use std::collections::HashMap;
use std::sync::Arc;

pub enum Expr {
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    RecordRead(Arc<str>),
    RecordReadFill(Arc<str>),
    RecordWrite(Arc<str>, Box<Expr>),
    RecordDelete(Arc<str>),
    Literal(Record),
    ArrayLiteral(Vec<Box<Expr>>),
    HashLiteral(HashMap<Arc<str>, Box<Expr>>),
}

pub enum UnaryOp {
    LogNeg(),
    NumNeg(),
}

pub enum BinaryOp {
    LogOr(),
    LogAnd(),

    NumLt(),
    NumLte(),
    NumGt(),
    NumGte(),
    NumEq(),
    NumNe(),

    Lt(),
    Lte(),
    Gt(),
    Gte(),
    Eq(),
    Ne(),

    Add(),
    Sub(),
    Cat(),

    Mul(),
    Div(),
    Mod(),
}

pub fn int_literal(s: &str) -> Box<Expr> {
    let n: i64 = s.parse().unwrap();
    return Box::new(Expr::Literal(Record::from(n)));
}

pub fn float_literal(s: &str) -> Box<Expr> {
    let n: f64 = s.parse().unwrap();
    return Box::new(Expr::Literal(Record::from(n)));
}

pub fn string_literal(s: &str) -> Box<Expr> {
    let s: Vec<_> = s.chars().collect();
    assert!(s[0] == '"');
    assert!(s[s.len() - 1] == '"');
    let s = &s[1..(s.len() - 1)];

    let mut i = s.iter();
    let mut s = "".to_string();
    while let Some(c) = i.next() {
        match c {
            '\\' => {
                match i.next() {
                    Some('t') => {
                        s.push('\t');
                    }
                    Some('n') => {
                        s.push('\n');
                    }
                    Some('\\') => {
                        s.push('\\');
                    }
                    Some('"') => {
                        s.push('"');
                    }
                    _ => {
                        panic!();
                    }
                }
            }
            c => {
                s.push(*c);
            }
        }
    }

    return Box::new(Expr::Literal(Record::from(s)));
}

pub fn path_literal(s: &str) -> Arc<str> {
    let s: Vec<_> = s.chars().collect();
    assert!(s[0] == '{');
    assert!(s[1] == '{');
    assert!(s[s.len() - 2] == '}');
    assert!(s[s.len() - 1] == '}');
    let s = &s[2..(s.len() - 2)];

    let s: String = s.into_iter().collect();
    return Arc::from(s);
}
