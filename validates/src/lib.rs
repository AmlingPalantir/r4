use std::error::Error;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub enum ValidationError {
    Message(String),
    Error(Box<Error>),
}

impl<E: Error + 'static> From<E> for ValidationError {
    fn from(e: E) -> ValidationError {
        return ValidationError::Error(Box::new(e));
    }
}

pub type ValidationResult<T> = Result<T, ValidationError>;

impl Debug for ValidationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        return match self {
            ValidationError::Message(msg) => write!(f, "{}", msg),
            ValidationError::Error(e) => e.fmt(f),
        };
    }
}

pub trait Validates {
    type Target;

    fn validate(self) -> ValidationResult<Self::Target>;
}
