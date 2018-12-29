use StreamWrapper;
use std::sync::Arc;
use validates::Validates;

#[derive(Default)]
pub struct SubOperationOption(Vec<String>);

impl SubOperationOption {
    pub fn push(&mut self, a: &[String]) {
        self.0.extend_from_slice(a);
    }

    pub fn of(a: Vec<String>) -> SubOperationOption {
        return SubOperationOption(a);
    }
}

impl Validates for SubOperationOption {
    type Target = SubOperationOptions;

    fn validate(mut self) -> SubOperationOptions {
        let name = self.0.remove(0);
        let op = super::REGISTRY.find(&name, &[]);
        let wr = op(&mut self.0);
        return SubOperationOptions {
            extra: self.0,
            wr: Arc::new(wr),
        };
    }
}

#[derive(Clone)]
pub struct SubOperationOptions {
    pub extra: Vec<String>,
    pub wr: Arc<StreamWrapper>,
}
