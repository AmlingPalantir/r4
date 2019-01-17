use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub struct ValidationError {
    pub msg: String,
}

pub type ValidationResult<T> = Result<T, ValidationError>;

impl Debug for ValidationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        return write!(f, "{}", self.msg);
    }
}

pub trait Validates {
    type Target;

    fn validate(self) -> ValidationResult<Self::Target>;
}
