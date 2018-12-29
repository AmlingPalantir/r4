use OperationBe2;
use opts::parser::OptParserView;
use stream::Stream;
use super::eval::InputType;
use super::eval::Options;
use super::eval::OptionsValidated;
use super::eval::OutputType;

pub struct Impl();

impl OperationBe2 for Impl {
    type Options = Options;
    type OptionsValidated = OptionsValidated;

    fn names() -> Vec<&'static str> {
        return vec!["xform"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        super::eval::Impl::options(opt);
    }

    fn stream(o: &OptionsValidated) -> Stream {
        return super::eval::stream1(o, InputType::Records(), OutputType::Records(), false);
    }
}
