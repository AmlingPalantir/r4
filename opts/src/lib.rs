use std::collections::HashMap;
use std::rc::Rc;

enum ExtraHandler<P> {
    Soft(Rc<Fn(&mut P, &String) -> bool>),
    Hard(Rc<Fn(&mut P, &[String])>),
}

pub struct OptParser<P> {
    named: HashMap<String, (usize, Rc<Fn(&mut P, &[String])>)>,
    extra: Vec<ExtraHandler<P>>,
}

pub struct OptParserView<'a, P: 'a, P2> {
    op: &'a mut OptParser<P>,
    f: Rc<Fn(&mut P) -> &mut P2>,
}

fn name_from_arg(name: &str) -> Option<&str> {
    if name.starts_with("--") {
        return Some(&name[2..]);
    }
    if name.starts_with("-") {
        return Some(&name[1..]);
    }
    return None;
}

impl<P: Default> OptParser<P> {
    pub fn new() -> OptParser<P> {
        return OptParser {
            named: HashMap::new(),
            extra: Vec::new(),
        };
    }

    pub fn parse(&self, args: &Vec<String>) -> P {
        let mut p = P::default();

        let mut next_index = 0;
        let mut refuse_opt = false;
        'arg: loop {
            if next_index == args.len() {
                return p;
            }

            if !refuse_opt {
                if &args[next_index] == "--" {
                    refuse_opt = true;
                    next_index += 1;
                    continue 'arg;
                }

                if let Some(name) = name_from_arg(&args[next_index]) {
                    match self.named.get(name) {
                        Some((argct, f)) => {
                            let start = next_index + 1;
                            let end = start + argct;
                            if end > args.len() {
                                panic!();
                            }
                            f(&mut p, &args[start..end]);
                            next_index = end;
                            continue;
                        }
                        None => {
                            panic!();
                        }
                    }
                }
            }

            for extra in &self.extra {
                match extra {
                    ExtraHandler::Soft(f) => {
                        if f(&mut p, &args[next_index]) {
                            next_index += 1;
                            continue 'arg;
                        }
                    }
                    ExtraHandler::Hard(f) => {
                        f(&mut p, &args[next_index..]);
                        next_index = args.len();
                        continue 'arg;
                    }
                }
            }

            panic!();
        }
    }
}

impl<P> OptParser<P> {
    pub fn view<'a>(&'a mut self) -> OptParserView<'a, P, P> {
        return OptParserView {
            op: self,
            f: Rc::new(|p| p),
        };
    }
}

impl<'a, P: 'static, P2: 'static> OptParserView<'a, P, P2> {
    pub fn sub<P3, F: Fn(&mut P2) -> &mut P3 + 'static>(&'a mut self, f: F) -> OptParserView<'a, P, P3> {
        let f1 = self.f.clone();
        return OptParserView::<'a, P, P3> {
            op: self.op,
            f: Rc::new(move |p| f(f1(p))),
        }
    }

    pub fn match_single<F: Fn(&mut P2, &String) + 'static>(&mut self, aliases: &[&str], f: F) {
        self.match_n(aliases, 1, move |p, a| f(p, &a[0]));
    }

    pub fn match_zero<F: Fn(&mut P2) + 'static>(&mut self, aliases: &[&str], f: F) {
        self.match_n(aliases, 0, move |p, _a| f(p));
    }

    pub fn match_n<F: Fn(&mut P2, &[String]) + 'static>(&mut self, aliases: &[&str], argct: usize, f: F) {
        let f1 = self.f.clone();
        let f: Rc<Fn(&mut P, &[String])> = Rc::new(move |p, a| f(f1(p), a));
        for alias in aliases {
            let prev = self.op.named.insert(alias.to_string(), (argct, f.clone()));
            if prev.is_some() {
                panic!();
            }
        }
    }

    pub fn match_extra_soft<F: Fn(&mut P2, &String) -> bool + 'static>(&mut self, f: F) {
        let f1 = self.f.clone();
        let f: Rc<Fn(&mut P, &String) -> bool> = Rc::new(move |p, a| f(f1(p), a));
        self.op.extra.push(ExtraHandler::Soft(f.clone()));
    }

    pub fn match_extra_hard<F: Fn(&mut P2, &[String]) + 'static>(&mut self, f: F) {
        let f1 = self.f.clone();
        let f: Rc<Fn(&mut P, &[String])> = Rc::new(move |p, a| f(f1(p), a));
        self.op.extra.push(ExtraHandler::Hard(f.clone()));
    }
}

pub enum OneOption {
}

impl OneOption {
    pub fn set_string_option(p: &mut Option<String>, a: &String) {
        if let Some(_) = *p {
            panic!();
        }
        *p = Some(a.clone());
    }

    pub fn push_string_vec(p: &mut Vec<String>, a: &String) {
        p.push(a.clone());
    }
}



pub trait Validates {
    type To;

    fn validate(self) -> Self::To;
}

#[macro_export]
macro_rules! declare_opts {
    {$($name:ident: $type:ty,)*} => {
        #[derive(Default)]
        pub struct PreOptions {
            $(
                $name: <$type as $crate::OptionTrait>::PreType,
            )*
        }

        impl $crate::Validates for PreOptions {
            type To = PostOptions;

            fn validate(self) -> PostOptions {
                return PostOptions {
                    $(
                        $name: <$type as $crate::OptionTrait>::validate(self.$name),
                    )*
                };
            }
        }

        pub struct PostOptions {
            $(
                $name: <$type as $crate::OptionTrait>::ValType,
            )*
        }
    }
}

pub trait OptionTrait {
    type PreType;
    type ValType;

    fn validate(Self::PreType) -> Self::ValType;
}

pub enum RequiredStringOption {
}

impl OptionTrait for RequiredStringOption {
    type PreType = Option<String>;
    type ValType = String;

    fn validate(p: Option<String>) -> String {
        return p.unwrap();
    }
}
