use misc::PointerRc;
use std::rc::Rc;
use super::trie::NameTrie;
use super::trie::NameTrieResult;
use validates::ValidationResult;

type CbMany<P> = PointerRc<Fn(&mut P, &[String]) -> ValidationResult<()>>;
type CbOne<P> = PointerRc<Fn(&mut P, &str) -> ValidationResult<bool>>;

enum ExtraHandler<P> {
    Soft(CbOne<P>),
    Hard(CbMany<P>),
}

trait OptParserMatch<P: 'static> {
    fn match_n(&mut self, aliases: &[&str], argct: usize, f: CbMany<P>);
    fn match_extra_soft(&mut self, f: CbOne<P>);
    fn match_extra_hard(&mut self, f: CbMany<P>);
}

pub struct OptParserView<'a, P: 'a>(Box<OptParserMatch<P> + 'a>);

impl<'a, P: 'static> OptParserView<'a, P> {
    pub fn match_single<F: Fn(&mut P, &str) -> ValidationResult<()> + 'static>(&mut self, aliases: &[&str], f: F) {
        self.match_n(aliases, 1, move |p, a| f(p, &a[0]));
    }

    pub fn match_zero<F: Fn(&mut P) -> ValidationResult<()> + 'static>(&mut self, aliases: &[&str], f: F) {
        self.match_n(aliases, 0, move |p, _a| f(p));
    }

    pub fn match_n<F: Fn(&mut P, &[String]) -> ValidationResult<()> + 'static>(&mut self, aliases: &[&str], argct: usize, f: F) {
        self.0.match_n(aliases, argct, PointerRc(Rc::new(f)));
    }

    pub fn match_extra_soft<F: Fn(&mut P, &str) -> ValidationResult<bool> + 'static>(&mut self, f: F) {
        self.0.match_extra_soft(PointerRc(Rc::new(f)));
    }

    pub fn match_extra_hard<F: Fn(&mut P, &[String]) -> ValidationResult<()> + 'static>(&mut self, f: F) {
        self.0.match_extra_hard(PointerRc(Rc::new(f)));
    }
}



#[derive(Default)]
pub struct OptParser<P> {
    named: NameTrie<(usize, CbMany<P>)>,
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
    pub fn view<'a>(&'a mut self) -> OptParserView<'a, P> {
        return OptParserView(Box::new(self));
    }

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
                        NameTrieResult::None() => panic!("No such option {}", name),
                        NameTrieResult::Unique(_, e) => e,
                        NameTrieResult::Collision(name1, name2) => panic!("Option {} is ambiguous (e.g.  {} and {})", name, name1, name2),
                    };
                    let start = next_index + 1;
                    let end = start + argct;
                    if end > args.len() {
                        panic!("Not enough arguments for {}", args[next_index]);
                    }
                    (f.0)(p, &args[start..end])?;
                    next_index = end;
                    continue;
                }
            }

            for extra in &self.extra {
                match extra {
                    ExtraHandler::Soft(f) => {
                        if (f.0)(p, &args[next_index])? {
                            next_index += 1;
                            continue 'arg;
                        }
                    }
                    ExtraHandler::Hard(f) => {
                        (f.0)(p, &args[next_index..])?;
                        next_index = args.len();
                        continue 'arg;
                    }
                }
            }

            panic!("No handler at {}", args[next_index]);
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

impl<'a, P: 'static> OptParserMatch<P> for &'a mut OptParser<P> {
    fn match_n(&mut self, aliases: &[&str], argct: usize, f: CbMany<P>) {
        for alias in aliases {
            self.named.insert(alias, (argct, f.clone()));
        }
    }

    fn match_extra_soft(&mut self, f: CbOne<P>) {
        self.extra.push(ExtraHandler::Soft(f));
    }

    fn match_extra_hard(&mut self, f: CbMany<P>) {
        self.extra.push(ExtraHandler::Hard(f));
    }
}



struct OptParserSubMatch<'a, PP: 'a, P> {
    parent: &'a mut OptParserMatch<PP>,
    f: Rc<Fn(&mut PP) -> &mut P>,
}

impl<'a, PP: 'static, P: 'static> OptParserMatch<P> for OptParserSubMatch<'a, PP, P> {
    fn match_n(&mut self, aliases: &[&str], argct: usize, f: CbMany<P>) {
        let f1 = self.f.clone();
        self.parent.match_n(aliases, argct, PointerRc(Rc::new(move |p, a| (f.0)(f1(p), a))));
    }

    fn match_extra_soft(&mut self, f: CbOne<P>) {
        let f1 = self.f.clone();
        self.parent.match_extra_soft(PointerRc(Rc::new(move |p, a| (f.0)(f1(p), a))));
    }

    fn match_extra_hard(&mut self, f: CbMany<P>) {
        let f1 = self.f.clone();
        self.parent.match_extra_hard(PointerRc(Rc::new(move |p, a| (f.0)(f1(p), a))));
    }
}

impl<'a, P: 'static> OptParserView<'a, P> {
    pub fn sub<'b, P2: 'static, F: Fn(&mut P) -> &mut P2 + 'static>(&'b mut self, f: F) -> OptParserView<'b, P2> where 'a: 'b {
        return OptParserView(Box::new(OptParserSubMatch {
            parent: &mut *self.0,
            f: Rc::new(f),
        }));
    }
}
