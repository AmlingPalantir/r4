use std::sync::Arc;
use super::StreamWrapper;
use validates::Validates;
use validates::ValidationError;
use validates::ValidationResult;

#[derive(Default)]
pub struct SubOperationOption(Vec<String>);

impl SubOperationOption {
    pub fn push(&mut self, a: &[String]) -> ValidationResult<()> {
        self.0.extend_from_slice(a);
        return Result::Ok(());
    }

    pub fn of(a: Vec<String>) -> SubOperationOption {
        return SubOperationOption(a);
    }
}

impl Validates for SubOperationOption {
    type Target = SubOperationOptionValidated;

    fn validate(mut self) -> ValidationResult<SubOperationOptionValidated> {
        if self.0.len() == 0 {
            return ValidationError::message("No sub-operation specified");
        }
        let name = self.0.remove(0);
        let op = super::REGISTRY.find(&name, &[])?;
        let wr = op.parse(&mut self.0);
        return Result::Ok(SubOperationOptionValidated {
            extra: self.0,
            wr: Arc::new(wr),
        });
    }
}

#[derive(Clone)]
pub struct SubOperationOptionValidated {
    pub extra: Vec<String>,
    pub wr: Arc<StreamWrapper>,
}
