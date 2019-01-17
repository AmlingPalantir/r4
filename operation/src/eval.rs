use executor::BoxedExecutor2;
use opts::parser::OptParserView;
use opts::vals::BooleanOption;
use opts::vals::DefaultedOption;
use opts::vals::OptionDefaulter;
use opts::vals::OptionalStringOption;
use opts::vals::RequiredStringOption;
use record::Record;
use record::RecordTrait;
use registry::Registrant;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use validates::Validates;
use validates::ValidationResult;

option_defaulters! {
    InputRecordsDefaulter: InputType => InputType::Records(),
    InputLinesDefaulter: InputType => InputType::Lines(),

    OutputRecordsDefaulter: OutputType => OutputType::Records(),
    OutputLinesDefaulter: OutputType => OutputType::Lines(),
    OutputGrepDefaulter: OutputType => OutputType::Grep(),

    FalseDefaulter: bool => false,
    TrueDefaulter: bool => true,
}

#[derive(Clone)]
pub enum InputType {
    Records(),
    Lines(),
}

#[derive(Clone)]
pub enum OutputType {
    Records(),
    Lines(),
    Grep(),
}

#[derive(Default)]
struct CodeOptions {
    engine: OptionalStringOption,
    code: RequiredStringOption,
}

impl Validates for CodeOptions {
    type Target = BoxedExecutor2;

    fn validate(self) -> ValidationResult<BoxedExecutor2> {
        let engine = self.engine.validate()?.unwrap_or_else(|| executor::r4l::Impl::names()[0].to_string());
        let executor = executor::REGISTRY.find(&engine, &[]);
        let executor = executor.parse(&self.code.validate()?)?;
        return Result::Ok(executor);
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct EvalOptions<I: OptionDefaulter<InputType>, O: OptionDefaulter<OutputType>, R: OptionDefaulter<bool>> {
    invert: BooleanOption,
    code: CodeOptions,
    input: DefaultedOption<InputType, I>,
    output: DefaultedOption<OutputType, O>,
    ret: DefaultedOption<bool, R>,
}

pub trait EvalBe {
    type I: OptionDefaulter<InputType> + Default;
    type O: OptionDefaulter<OutputType> + Default;
    type R: OptionDefaulter<bool> + Default;

    fn names() -> Vec<&'static str>;
}

pub struct EvalBe2<B: EvalBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: EvalBe + 'static> OperationBe2 for EvalBe2<B> {
    type Options = EvalOptions<B::I, B::O, B::R>;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn options<'a>(opt: &mut OptParserView<'a, Self::Options>) {
        opt.sub(|p| &mut p.invert).match_zero(&["v", "invert"], BooleanOption::set);
        opt.sub(|p| &mut p.invert).match_zero(&["no-invert"], BooleanOption::clear);
        opt.sub(|p| &mut p.code.code).match_extra_soft(RequiredStringOption::maybe_set_str);
        opt.sub(|p| &mut p.code.engine).match_single(&["engine"], OptionalStringOption::set_str);
        opt.match_zero(&["lua"], |p| p.code.engine.set("lua".to_string()));
        opt.match_zero(&["input-lines"], |p| p.input.set(InputType::Lines()));
        opt.match_zero(&["input-records"], |p| p.input.set(InputType::Records()));
        opt.match_zero(&["output-lines"], |p| p.output.set(OutputType::Lines()));
        opt.match_zero(&["output-records"], |p| p.output.set(OutputType::Records()));
        opt.match_zero(&["output-grep"], |p| p.output.set(OutputType::Grep()));
        opt.match_zero(&["return"], |p| p.ret.set(true));
        opt.match_zero(&["no-return"], |p| p.ret.set(false));
    }

    fn stream(o: Arc<EvalOptionsValidated<B::I, B::O, B::R>>) -> Stream {
        let f: Box<FnMut(Record) -> Record>;
        f = o.code.stream(o.ret);

        return stream::closures(
            f,
            move |s, e, w| {
                let ri;
                match e.clone() {
                    Entry::Bof(file) => {
                        return w(Entry::Bof(file));
                    }
                    Entry::Record(r) => {
                        ri = match o.input {
                            InputType::Records() => r,
                            InputType::Lines() => Record::from(r.deparse()),
                        };
                    }
                    Entry::Line(line) => {
                        ri = match o.input {
                            InputType::Records() => Record::parse(&line),
                            InputType::Lines() => Record::from(line),
                        };
                    }
                }
                let ro = s(ri);
                let ro = if o.invert { Record::from(!ro.coerce_bool()) } else { ro };
                return match o.output {
                    OutputType::Records() => w(Entry::Record(ro)),
                    OutputType::Lines() => w(Entry::Line(ro.coerce_string())),
                    OutputType::Grep() => !ro.coerce_bool() || w(e),
                };
            },
            |_s, _w| {
            },
        );
    }
}

pub type EvalImpl<B> = OperationRegistrant<OperationBeForBe2<EvalBe2<B>>>;

pub enum EvalBeImpl {
}

impl EvalBe for EvalBeImpl {
    type I = InputRecordsDefaulter;
    type O = OutputLinesDefaulter;
    type R = TrueDefaulter;

    fn names() -> Vec<&'static str> {
        return vec!["eval"];
    }
}

pub type Impl = EvalImpl<EvalBeImpl>;
