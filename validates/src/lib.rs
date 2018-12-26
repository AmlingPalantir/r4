pub trait Validates {
    type Target;

    fn validate(self) -> Self::Target;
}
