use std::error::Error;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::ops::Deref;

pub enum ValidationError {
    Message(String),
    Error(Box<Error>),
}

impl<E: Error + 'static> From<E> for ValidationError {
    fn from(e: E) -> ValidationError {
        return ValidationError::Error(Box::new(e));
    }
}

impl ValidationError {
    pub fn message<R>(msg: String) -> ValidationResult<R> {
        return Result::Err(ValidationError::Message(msg));
    }

    pub fn label<S: Deref<Target = str>>(&self, prefix: S) -> ValidationError {
        return ValidationError::Message(format!("{}: {:?}", &*prefix, self));
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
