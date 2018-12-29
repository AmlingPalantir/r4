use super::eval::EvalBe;
use super::eval::EvalImpl;
use super::eval::InputRecordsDefaulter;
use super::eval::OutputRecordsDefaulter;
use super::eval::FalseDefaulter;

pub enum EvalBeImpl {
}

impl EvalBe for EvalBeImpl {
    type I = InputRecordsDefaulter;
    type O = OutputRecordsDefaulter;
    type R = FalseDefaulter;

    fn names() -> Vec<&'static str> {
        return vec!["xform"];
    }
}

pub type Impl = EvalImpl<EvalBeImpl>;
