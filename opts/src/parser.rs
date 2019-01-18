use misc::PointerRc;
use std::rc::Rc;
use super::trie::NameTrie;
use super::trie::NameTrieResult;
use validates::Validates;
use validates::ValidationError;
use validates::ValidationResult;

type CbMany<P> = PointerRc<Fn(&mut P, &[String]) -> ValidationResult<()>>;
type CbOne<P> = PointerRc<Fn(&mut P, &str) -> ValidationResult<bool>>;

enum ExtraHandler<P> {
    Soft(CbOne<P>),
    Hard(CbMany<P>),
}

enum OptionsPileElement<P> {
    Args(Vec<String>, usize, CbMany<P>),
    Extra(ExtraHandler<P>),
}

pub struct OptionsPile<P>(Vec<OptionsPileElement<P>>);

impl<P: 'static> OptionsPile<P> {
    pub fn new() -> Self {
        return OptionsPile(Vec::new());
    }

    pub fn match_single<F: Fn(&mut P, &str) -> ValidationResult<()> + 'static>(&mut self, aliases: &[&str], f: F) {
        self.match_n(aliases, 1, move |p, a| f(p, &a[0]));
    }

    pub fn match_zero<F: Fn(&mut P) -> ValidationResult<()> + 'static>(&mut self, aliases: &[&str], f: F) {
        self.match_n(aliases, 0, move |p, _a| f(p));
    }

    pub fn match_n<F: Fn(&mut P, &[String]) -> ValidationResult<()> + 'static>(&mut self, aliases: &[&str], argct: usize, f: F) {
        self.0.push(OptionsPileElement::Args(aliases.iter().map(|s| s.to_string()).collect(), argct, PointerRc(Rc::new(f))));
    }

    pub fn match_extra_soft<F: Fn(&mut P, &str) -> ValidationResult<bool> + 'static>(&mut self, f: F) {
        self.0.push(OptionsPileElement::Extra(ExtraHandler::Soft(PointerRc(Rc::new(f)))));
    }

    pub fn match_extra_hard<F: Fn(&mut P, &[String]) -> ValidationResult<()> + 'static>(&mut self, f: F) {
        self.0.push(OptionsPileElement::Extra(ExtraHandler::Hard(PointerRc(Rc::new(f)))));
    }

    pub fn add(&mut self, mut other: OptionsPile<P>) {
        self.0.append(&mut other.0);
    }

    pub fn add_sub<P2: 'static, F: Fn(&mut P) -> &mut P2 + 'static>(&mut self, f: F, other: OptionsPile<P2>) {
        self.add(other.sub(f));
    }

    pub fn to_parser(&self) -> OptParser<P> {
        let mut opt = OptParser::default();
        for e in self.0.iter() {
            match e {
                OptionsPileElement::Args(aliases, argct, f) => {
                    for alias in aliases {
                        opt.named.insert(&alias, (*argct, f.clone()));
                    }
                }
                OptionsPileElement::Extra(ExtraHandler::Soft(h)) => {
                    opt.extra.push(ExtraHandler::Soft(h.clone()));
                }
                OptionsPileElement::Extra(ExtraHandler::Hard(h)) => {
                    opt.extra.push(ExtraHandler::Hard(h.clone()));
                }
            }
        }
        return opt;
    }

    pub fn sub<P2, F: Fn(&mut P2) -> &mut P + 'static>(self, f1: F) -> OptionsPile<P2> {
        let f1 = Rc::new(f1);
        return OptionsPile(self.0.into_iter().map(|e| {
            let f1 = f1.clone();
            return match e {
                OptionsPileElement::Args(aliases, argct, f) => OptionsPileElement::Args(aliases, argct, PointerRc(Rc::new(move |p, a| (f.0)(f1(p), a)))),
                OptionsPileElement::Extra(ExtraHandler::Soft(h)) => OptionsPileElement::Extra(ExtraHandler::Soft(PointerRc(Rc::new(move |p, a| (h.0)(f1(p), a))))),
                OptionsPileElement::Extra(ExtraHandler::Hard(h)) => OptionsPileElement::Extra(ExtraHandler::Hard(PointerRc(Rc::new(move |p, a| (h.0)(f1(p), a))))),
            };
        }).collect());
    }
}



pub trait Optionsable {
    type Options: Default + Validates + 'static;

    fn options(opt: &mut OptionsPile<Self::Options>);

    fn new_options() -> OptionsPile<Self::Options> {
        let mut opt = OptionsPile::new();
        Self::options(&mut opt);
        return opt;
    }
}



pub struct OptParser<P> {
    named: NameTrie<(usize, CbMany<P>)>,
    extra: Vec<ExtraHandler<P>>,
}

impl<P> Default for OptParser<P> {
    fn default() -> Self {
        return OptParser {
            named: NameTrie::default(),
            extra: Vec::default(),
        };
    }
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
    pub fn parse_mut(&self, args: &[String], p: &mut P) -> ValidationResult<()> {
        let mut next_index = 0;
        let mut refuse_opt = false;
        'arg: loop {
            if next_index == args.len() {
                return Result::Ok(());
            }

            if !refuse_opt {
                if &args[next_index] == "--" {
                    refuse_opt = true;
                    next_index += 1;
                    continue 'arg;
                }

                if let Some(name) = name_from_arg(&args[next_index]) {
                    let (argct, f) = match self.named.get(name) {
                        NameTrieResult::None() => return ValidationError::message(format!("No such option {}", name)),
                        NameTrieResult::Unique(_, e) => e,
                        NameTrieResult::Collision(name1, name2) => return ValidationError::message(format!("Option {} is ambiguous (e.g.  {} and {})", name, name1, name2)),
                    };
                    let start = next_index + 1;
                    let end = start + argct;
                    if end > args.len() {
                        return ValidationError::message(format!("Not enough arguments for {}", args[next_index]));
                    }
                    (f.0)(p, &args[start..end]).map_err(|e| e.label(format!("While handline {:?}", &args[next_index..end])))?;
                    next_index = end;
                    continue;
                }
            }

            for extra in &self.extra {
                match extra {
                    ExtraHandler::Soft(f) => {
                        if (f.0)(p, &args[next_index]).map_err(|e| e.label(format!("While handling {:?}: {:?}", &args[next_index..=next_index], e)))? {
                            next_index += 1;
                            continue 'arg;
                        }
                    }
                    ExtraHandler::Hard(f) => {
                        (f.0)(p, &args[next_index..]).map_err(|e| e.label(format!("While handline {:?}: {:?}", &args[next_index..], e)))?;
                        next_index = args.len();
                        continue 'arg;
                    }
                }
            }

            return ValidationError::message(format!("Unexpected extra arguments: {:?}", &args[next_index..]));
        }
    }
}

impl<P: Default + 'static> OptParser<P> {
    pub fn parse(&self, args: &[String]) -> ValidationResult<P> {
        let mut p = P::default();
        self.parse_mut(args, &mut p)?;
        return Result::Ok(p);
    }
}
