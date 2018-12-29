use OperationBe2;
use executor::Code;
use opts::parser::OptParserView;
use opts::vals::BooleanOption;
use opts::vals::OptionalOption;
use opts::vals::RequiredOption;
use record::Record;
use record::RecordTrait;
use std::ops::Deref;
use stream::Entry;
use stream::Stream;
use validates::Validates;

pub struct Impl();

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
#[derive(Validates)]
pub struct Options {
    invert: BooleanOption,
    code: RequiredOption<Code>,
    input: OptionalOption<InputType>,
    output: OptionalOption<OutputType>,
    ret: OptionalOption<bool>,
}

impl OperationBe2 for Impl {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["eval"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.invert).match_zero(&["v", "invert"], BooleanOption::set);
        opt.sub(|p| &mut p.invert).match_zero(&["no-invert"], BooleanOption::clear);
        opt.match_extra_soft(|p, a| p.code.maybe_set_with(|| Code::parse(a)));
        opt.match_zero(&["input-lines"], |p| p.input.set(InputType::Lines()));
        opt.match_zero(&["input-records"], |p| p.input.set(InputType::Records()));
        opt.match_zero(&["output-lines"], |p| p.output.set(OutputType::Lines()));
        opt.match_zero(&["output-records"], |p| p.output.set(OutputType::Records()));
        opt.match_zero(&["output-grep"], |p| p.output.set(OutputType::Grep()));
        opt.match_zero(&["return"], |p| p.ret.set(true));
        opt.match_zero(&["no-return"], |p| p.ret.set(false));
    }

    fn stream(o: impl Deref<Target = OptionsValidated>) -> Stream {
        return stream1(o, InputType::Records(), OutputType::Lines(), true);
    }
}

pub fn stream1(o: impl Deref<Target = OptionsValidated>, def_input: InputType, def_output: OutputType, def_ret: bool) -> Stream {
    let invert = o.invert;
    let input = o.input.clone().unwrap_or(def_input);
    let output = o.output.clone().unwrap_or(def_output);
    let ret = o.ret.clone().unwrap_or(def_ret);

    return stream::closures(
        o.code.clone().stream(),
        move |s, e, w| {
            let ri;
            match e.clone() {
                Entry::Bof(file) => {
                    return w(Entry::Bof(file));
                }
                Entry::Record(r) => {
                    ri = match input {
                        InputType::Records() => r,
                        InputType::Lines() => Record::from(r.deparse()),
                    };
                }
                Entry::Line(line) => {
                    ri = match input {
                        InputType::Records() => Record::parse(&line),
                        InputType::Lines() => Record::from(line),
                    };
                }
            }
            let (rr, ro) = s(ri);
            let ro = if ret { rr } else { ro };
            let ro = if invert { Record::from(!ro.coerce_bool()) } else { ro };
            return match output {
                OutputType::Records() => w(Entry::Record(ro)),
                OutputType::Lines() => w(Entry::Line(ro.coerce_string())),
                OutputType::Grep() => !ro.coerce_bool() || w(e),
            };
        },
        |_s, _w| {
        },
    );
}
