use std::sync::Arc;
use std::vec::Vec;

pub enum PathStep<'a> {
    RefHash(&'a str),
    OwnHash(Arc<str>),
    Array(usize),
}

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
