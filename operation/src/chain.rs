use opts::parser::OptParserView;
use opts::vals::BooleanOption;
use std::sync::Arc;
use stream::Stream;
use super::OperationBe;
use super::OperationRegistrant;
use super::StreamWrapper;
use super::SubOperationOption;
use validates::Validates;

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
        opt.sub(|p| &mut p.cmds).match_extra_hard(CmdsOption::push);
        opt.sub(|p| &mut p.keep_bof).match_zero(&["keep-bof"], BooleanOption::set);
        opt.sub(|p| &mut p.keep_bof).match_zero(&["no-keep-bof"], BooleanOption::clear);
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return o.cmds.extra.clone();
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return o.cmds.wrs.iter().rev().fold(stream::id(), |mut s, wr| {
            if !o.keep_bof {
                s = stream::compound(stream::drop_bof(), s);
            }
            return stream::compound(wr.stream(), s)
        });
    }
}

#[derive(Default)]
struct CmdsOption(Vec<String>);

impl Validates for CmdsOption {
    type Target = CmdsOptions;

    fn validate(self) -> CmdsOptions {
        let mut iter = self.0.into_iter();
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
                        if first == "|" {
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
            let so = SubOperationOption::of(cmd).validate();
            match extra {
                None => {
                    extra = Some(so.extra);
                }
                Some(_) => {
                    if !so.extra.is_empty() {
                        panic!("Unexpected extra args for non-first chain stage?");
                    }
                }
            }
            wrs.push(so.wr);
        }

        return CmdsOptions {
            extra: extra.unwrap(),
            wrs: wrs,
        };
    }
}

impl CmdsOption {
    fn push(&mut self, a: &[String]) {
        self.0.extend_from_slice(a);
    }
}

#[derive(Clone)]
struct CmdsOptions {
    extra: Vec<String>,
    wrs: Vec<Arc<StreamWrapper>>,
}
