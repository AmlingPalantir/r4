use super::eval::EvalBe;
use super::eval::EvalImpl;
use super::eval::InputRecordsDefaulter;
use super::eval::OutputGrepDefaulter;
use super::eval::TrueDefaulter;

pub enum EvalBeImpl {
}

impl EvalBe for EvalBeImpl {
    type I = InputRecordsDefaulter;
    type O = OutputGrepDefaulter;
    type R = TrueDefaulter;

    fn names() -> Vec<&'static str> {
        return vec!["grep"];
    }
}

pub type Impl = EvalImpl<EvalBeImpl>;
