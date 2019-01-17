use opts::parser::OptParserView;
use opts::vals::BooleanOption;
use opts::vals::OptionalStringOption;
use std::sync::Arc;
use stream::Stream;
use super::OperationBe;
use super::OperationRegistrant;
use super::StreamWrapper;
use super::SubOperationOption;
use validates::Validates;
use validates::ValidationError;
use validates::ValidationResult;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    keep_bof: BooleanOption,
    cmds: CmdsOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl OperationBe for ImplBe {
    type Options = Options;

    fn names() -> Vec<&'static str> {
        return vec!["chain"];
    }

    fn options<'a>(opt: &mut OptParserView<'a, Options>) {
        opt.sub(|p| &mut p.cmds.delim).match_single(&["d", "delim"], OptionalStringOption::set_str);
        opt.match_extra_hard(|p, a| {
            p.cmds.args.extend_from_slice(a);
            return Result::Ok(());
        });
        opt.sub(|p| &mut p.keep_bof).match_zero(&["keep-bof"], BooleanOption::set);
        opt.sub(|p| &mut p.keep_bof).match_zero(&["no-keep-bof"], BooleanOption::clear);
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return o.cmds.extra.clone();
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return o.cmds.wrs.iter().rev().fold((true, stream::id()), |(first, mut s), wr| {
            if !first && !o.keep_bof {
                s = stream::compound(stream::drop_bof(), s);
            }
            return (false, stream::compound(wr.stream(), s));
        }).1;
    }
}

#[derive(Default)]
struct CmdsOption {
    delim: OptionalStringOption,
    args: Vec<String>,
}

impl Validates for CmdsOption {
    type Target = CmdsOptions;

    fn validate(self) -> ValidationResult<CmdsOptions> {
        let delim = self.delim.validate()?.unwrap_or("|".to_string());
        let mut iter = self.args.into_iter();
        let mut cmds = Vec::new();
        'TOP: loop {
            let mut cmd = Vec::new();
            loop {
                match iter.next() {
                    None => {
                        cmds.push(cmd);
                        break 'TOP;
                    }
                    Some(first) => {
                        if first == delim {
                            cmds.push(cmd);
                            continue 'TOP;
                        }
                        cmd.push(first);
                    }
                }
            }
        }

        let mut extra = None;
        let mut wrs = Vec::new();
        for cmd in cmds {
            let so = SubOperationOption::of(cmd).validate()?;
            match extra {
                None => {
                    extra = Some(so.extra);
                }
                Some(_) => {
                    if !so.extra.is_empty() {
                        return ValidationError::message(format!("Unexpected extra args for non-first chain stage: {:?}", so.extra));
                    }
                }
            }
            wrs.push(so.wr);
        }

        return Result::Ok(CmdsOptions {
            extra: extra.unwrap(),
            wrs: wrs,
        });
    }
}

#[derive(Clone)]
struct CmdsOptions {
    extra: Vec<String>,
    wrs: Vec<Arc<StreamWrapper>>,
}
