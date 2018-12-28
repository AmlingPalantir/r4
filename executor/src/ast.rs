use record::Record;
use std::collections::HashMap;
use std::sync::Arc;
use record::Path;
use record::OwnPath;

pub enum Expr {
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    RecordRead(Box<Expr>, OwnPath),
    RecordReadFill(Box<Expr>, OwnPath),
    RecordWrite(Box<Expr>, OwnPath, Box<Expr>),
    RecordDelete(Box<Expr>, OwnPath),
    Literal(Record),
    ArrayLiteral(Vec<Box<Expr>>),
    HashLiteral(HashMap<Arc<str>, Box<Expr>>),
    WriteVar(Arc<str>, Box<Expr>),
    ReadVar(Arc<str>),
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

pub fn path_literal(s: &str) -> (Arc<str>, OwnPath) {
    // Arggh rust, you are so very very bad at string manipulation...
    let s: Vec<_> = s.chars().collect();
    assert!(s[0] == '{');
    assert!(s[1] == '{');
    assert!(s[s.len() - 2] == '}');
    assert!(s[s.len() - 1] == '}');
    let mut s = &s[2..(s.len() - 2)];

    let mut var = Arc::from("r");
    let i = s.iter().enumerate().find(|e| *e.1 == ':').map(|e| e.0);
    if let Some(i) = i {
        let name: String = (&s[0..i]).iter().cloned().collect();
        var = Arc::from(&name as &str);
        s = &s[(i + 1)..];
    }

    let s: String = s.into_iter().collect();
    return (var, Path::new(&s).to_owned());
}
