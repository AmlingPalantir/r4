use std::rc::Rc;
use super::trie::NameTrie;

enum ExtraHandler<P> {
    Soft(Rc<Fn(&mut P, &str) -> bool>),
    Hard(Rc<Fn(&mut P, &[String])>),
}

trait OptParserMatch<P: 'static> {
    fn match_n(&mut self, aliases: &[&str], argct: usize, f: Rc<Fn(&mut P, &[String])>);
    fn match_extra_soft(&mut self, f: Rc<Fn(&mut P, &str) -> bool>);
    fn match_extra_hard(&mut self, f: Rc<Fn(&mut P, &[String])>);
}

pub struct OptParserView<'a, P: 'a>(Box<OptParserMatch<P> + 'a>);

impl<'a, P: 'static> OptParserView<'a, P> {
    pub fn match_single<F: Fn(&mut P, &str) + 'static>(&mut self, aliases: &[&str], f: F) {
        self.match_n(aliases, 1, move |p, a| f(p, &a[0]));
    }

    pub fn match_zero<F: Fn(&mut P) + 'static>(&mut self, aliases: &[&str], f: F) {
        self.match_n(aliases, 0, move |p, _a| f(p));
    }

    pub fn match_n<F: Fn(&mut P, &[String]) + 'static>(&mut self, aliases: &[&str], argct: usize, f: F) {
        self.0.match_n(aliases, argct, Rc::new(f));
    }

    pub fn match_extra_soft<F: Fn(&mut P, &str) -> bool + 'static>(&mut self, f: F) {
        self.0.match_extra_soft(Rc::new(f));
    }

    pub fn match_extra_hard<F: Fn(&mut P, &[String]) + 'static>(&mut self, f: F) {
        self.0.match_extra_hard(Rc::new(f));
    }
}



#[derive(Default)]
pub struct OptParser<P> {
    named: NameTrie<(String, usize, Rc<Fn(&mut P, &[String])>)>,
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

    pub fn parse_mut(&self, args: &[String], p: &mut P) {
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
                    let (_, argct, f) = self.named.get(name).iter().fold(None, |hit, (name2, argct2, f2)| {
                        if let Some((name1, _, f1)) = hit {
                            if !Rc::ptr_eq(f1, f2) {
                                panic!("Option {} is ambiguous (e.g.  {} and {})", name, name1, name2);
                            }
                        }
                        return Some((name2, argct2, f2));
                    }).unwrap_or_else(|| panic!("No such option {}", name));
                    let start = next_index + 1;
                    let end = start + argct;
                    if end > args.len() {
                        panic!("Not enough arguments for {}", args[next_index]);
                    }
                    f(p, &args[start..end]);
                    next_index = end;
                    continue;
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

            panic!("No handler at {}", args[next_index]);
        }
    }
}

impl<P: Default + 'static> OptParser<P> {
    pub fn parse(&self, args: &[String]) -> P {
        let mut p = P::default();
        self.parse_mut(args, &mut p);
        return p;
    }
}

impl<'a, P: 'static> OptParserMatch<P> for &'a mut OptParser<P> {
    fn match_n(&mut self, aliases: &[&str], argct: usize, f: Rc<Fn(&mut P, &[String])>) {
        for alias in aliases {
            self.named.insert(alias, (alias.to_string(), argct, f.clone()));
        }
    }

    fn match_extra_soft(&mut self, f: Rc<Fn(&mut P, &str) -> bool>) {
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

    fn match_extra_soft(&mut self, f: Rc<Fn(&mut P, &str) -> bool>) {
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
        return OptParserView(Box::new(OptParserSubMatch {
            parent: &mut *self.0,
            f: Rc::new(f),
        }));
    }
}
