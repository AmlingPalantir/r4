use std::error::Error;
use std::ops::Deref;

pub enum ValidationError {
    Message(String),
}

impl<E: Error + 'static> From<E> for ValidationError {
    fn from(e: E) -> ValidationError {
        return ValidationError::Message(format!("{:?}", e));
    }
}

impl ValidationError {
    pub fn message<S: Deref<Target = str>, R>(msg: S) -> ValidationResult<R> {
        return Result::Err(ValidationError::Message(msg.to_string()));
    }

    pub fn label<S: Deref<Target = str>>(self, prefix: S) -> ValidationError {
        return match self {
            ValidationError::Message(s) => ValidationError::Message(format!("{}: {:?}", &*prefix, s)),
        };
    }

    pub fn panic(&self) -> ! {
        match self {
            ValidationError::Message(s) => panic!("{}", s),
        }
    }
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub trait Validates {
    type Target;

    fn validate(self) -> ValidationResult<Self::Target>;
}
