use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::OptionalStringOption;
use std::sync::Arc;
use stream::Stream;
use super::OperationBe;
use super::OperationRegistrant;
use validates::Validates;
use validates::ValidationError;
use validates::ValidationResult;

#[derive(Default)]
pub struct Options {
    name: OptionalStringOption,
}

pub enum Void {
}

impl Validates for Options {
    type Target = Void;

    fn validate(self) -> ValidationResult<Void> {
        let name = self.name.validate()?;
        return ValidationError::help(match name {
            Some(name) => super::REGISTRY.find(&name, &[])?.help(),
            None => super::REGISTRY.help_list(),
        });
    }
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl Optionsable for ImplBe {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_extra_soft(|p, a| p.name.maybe_set_str(a), "operation to show help for");
    }
}

impl OperationBe for ImplBe {
    fn names() -> Vec<&'static str> {
        return vec!["help"];
    }

    fn help_msg() -> &'static str {
        return "show help for an operation";
    }

    fn get_extra(_o: Arc<Void>) -> Vec<String> {
        panic!();
    }

    fn stream(_o: Arc<Void>) -> Stream {
        panic!();
    }
}
