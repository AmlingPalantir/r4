use std::sync::Arc;
use std::vec::Vec;

#[derive(Debug)]
pub enum PathStep<'a> {
    RefHash(&'a str),
    OwnHash(Arc<str>),
    Array(usize),
}

pub(crate) enum RPathStep<'a> {
    Hash(&'a str),
    Array(usize),
}

impl<'a> PathStep<'a> {
    pub(crate) fn as_r<'b>(&'b self) -> RPathStep<'b> where 'a: 'b {
        return match self {
            PathStep::RefHash(s) => RPathStep::Hash(s),
            PathStep::OwnHash(s) => RPathStep::Hash(s),
            PathStep::Array(n) => RPathStep::Array(*n),
        };
    }
}

#[derive(Debug)]
pub struct Path<'a>(pub(crate) Vec<PathStep<'a>>);
pub type OwnPath = Path<'static>;

impl<'a> Path<'a> {
    pub fn new(s: &'a str) -> Path<'a> {
        // arggh, split...
        if s.is_empty() {
            return Path(vec![]);
        }

        return Path(s.split('/').map(|e| {
            if e.starts_with('#') {
                return PathStep::Array(e[1..].parse().unwrap());
            }
            return PathStep::RefHash(e);
        }).collect());
    }

    pub fn to_owned(self) -> OwnPath {
        return Path(self.0.into_iter().map(|e| {
            return match e {
                PathStep::RefHash(s) => PathStep::OwnHash(Arc::from(s)),
                PathStep::OwnHash(s) => PathStep::OwnHash(s),
                PathStep::Array(n) => PathStep::Array(n),
            };
        }).collect());
    }
}
