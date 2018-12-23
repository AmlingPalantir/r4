use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::rc::Rc;

enum ExtraHandler<P> {
    Soft(Rc<Fn(&mut P, &String) -> bool>),
    Hard(Rc<Fn(&mut P, &[String])>),
}

trait OptParserMatch<P: 'static> {
    fn match_single(&mut self, aliases: &[&str], f: Rc<Fn(&mut P, &String)>) {
        self.match_n(aliases, 1, Rc::new(move |p, a| f(p, &a[0])));
    }

    fn match_zero(&mut self, aliases: &[&str], f: Rc<Fn(&mut P)>) {
        self.match_n(aliases, 0, Rc::new(move |p, _a| f(p)));
    }

    fn match_n(&mut self, &[&str], usize, Rc<Fn(&mut P, &[String])>);
    fn match_extra_soft(&mut self, Rc<Fn(&mut P, &String) -> bool>);
    fn match_extra_hard(&mut self, Rc<Fn(&mut P, &[String])>);
}

pub struct OptParserView<'a, P: 'a>(Box<OptParserMatch<P> + 'a>);

impl<'a, P: 'static> OptParserView<'a, P> {
    pub fn match_single<F: Fn(&mut P, &String) + 'static>(&mut self, aliases: &[&str], f: F) {
        self.0.match_single(aliases, Rc::new(f));
    }

    pub fn match_zero<F: Fn(&mut P) + 'static>(&mut self, aliases: &[&str], f: F) {
        self.0.match_zero(aliases, Rc::new(f));
    }

    pub fn match_n<F: Fn(&mut P, &[String]) + 'static>(&mut self, aliases: &[&str], argct: usize, f: F) {
        self.0.match_n(aliases, argct, Rc::new(f));
    }

    pub fn match_extra_soft<F: Fn(&mut P, &String) -> bool + 'static>(&mut self, f: F) {
        self.0.match_extra_soft(Rc::new(f));
    }

    pub fn match_extra_hard<F: Fn(&mut P, &[String]) + 'static>(&mut self, f: F) {
        self.0.match_extra_hard(Rc::new(f));
    }
}



pub struct OptParser<P> {
    named: HashMap<String, (usize, Rc<Fn(&mut P, &[String])>)>,
    extra: Vec<ExtraHandler<P>>,
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

impl<P: 'static> OptParser<P> {
    pub fn new() -> OptParser<P> {
        return OptParser {
            named: HashMap::new(),
            extra: Vec::new(),
        };
    }

    pub fn view<'a>(&'a mut self) -> OptParserView<'a, P> {
        return OptParserView(Box::new(self));
    }

    pub fn parse_mut(&self, args: &Vec<String>, p: &mut P) {
        let mut next_index = 0;
        let mut refuse_opt = false;
        'arg: loop {
            if next_index == args.len() {
                return;
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
                            f(p, &args[start..end]);
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
                        if f(p, &args[next_index]) {
                            next_index += 1;
                            continue 'arg;
                        }
                    }
                    ExtraHandler::Hard(f) => {
                        f(p, &args[next_index..]);
                        next_index = args.len();
                        continue 'arg;
                    }
                }
            }

            panic!();
        }
    }
}

impl<P: Default + 'static> OptParser<P> {
    pub fn parse(&self, args: &Vec<String>) -> P {
        let mut p = P::default();
        self.parse_mut(args, &mut p);
        return p;
    }
}

impl<'a, P: 'static> OptParserMatch<P> for &'a mut OptParser<P> {
    fn match_n(&mut self, aliases: &[&str], argct: usize, f: Rc<Fn(&mut P, &[String])>) {
        for alias in aliases {
            let prev = self.named.insert(alias.to_string(), (argct, f.clone()));
            if prev.is_some() {
                panic!();
            }
        }
    }

    fn match_extra_soft(&mut self, f: Rc<Fn(&mut P, &String) -> bool>) {
        self.extra.push(ExtraHandler::Soft(f));
    }

    fn match_extra_hard(&mut self, f: Rc<Fn(&mut P, &[String])>) {
        self.extra.push(ExtraHandler::Hard(f));
    }
}



struct OptParserSubMatch<'a, PP: 'a, P> {
    parent: &'a mut OptParserMatch<PP>,
    f: Rc<Fn(&mut PP) -> &mut P>,
}

impl<'a, PP: 'static, P: 'static> OptParserMatch<P> for OptParserSubMatch<'a, PP, P> {
    fn match_n(&mut self, aliases: &[&str], argct: usize, f: Rc<Fn(&mut P, &[String])>) {
        let f1 = self.f.clone();
        self.parent.match_n(aliases, argct, Rc::new(move |p, a| f(f1(p), a)));
    }

    fn match_extra_soft(&mut self, f: Rc<Fn(&mut P, &String) -> bool>) {
        let f1 = self.f.clone();
        self.parent.match_extra_soft(Rc::new(move |p, a| f(f1(p), a)));
    }

    fn match_extra_hard(&mut self, f: Rc<Fn(&mut P, &[String])>) {
        let f1 = self.f.clone();
        self.parent.match_extra_hard(Rc::new(move |p, a| f(f1(p), a)));
    }
}

impl<'a, P: 'static> OptParserView<'a, P> {
    pub fn sub<'b, P2: 'static, F: Fn(&mut P) -> &mut P2 + 'static>(&'b mut self, f: F) -> OptParserView<'b, P2> where 'a: 'b {
        // Unfortunately rustc can't seem to figure this one out without at
        // least some intermediate.  This was the least I could get it to work
        // with.
        let parent: &'b mut (OptParserMatch<P> + 'a) = self.0.borrow_mut();
        return OptParserView(Box::new(OptParserSubMatch {
            parent: parent,
            f: Rc::new(f),
        }));
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

pub enum VarOption {
}

impl VarOption {
    pub fn push_string_vec(p: &mut Vec<String>, a: &[String]) {
        p.extend_from_slice(a);
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

pub enum StringVecOption {
}

impl OptionTrait for StringVecOption {
    type PreType = Vec<String>;
    type ValType = Vec<String>;

    fn validate(p: Vec<String>) -> Vec<String> {
        return p;
    }
}
