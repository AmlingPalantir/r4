use std::borrow::BorrowMut;
use std::rc::Rc;
use super::trie::NameTrie;

enum ExtraHandler<P> {
    Soft(Rc<Fn(&mut P, &str) -> bool>),
    Hard(Rc<Fn(&mut P, &[String])>),
}

trait OptParserMatch<P: 'static> {
    fn match_n(&mut self, aliases: &str, id: Rc<()>, argct: usize, f: Rc<Fn(&mut P, &[String])>);
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

    pub fn match_n<S: AsRef<str>, I: IntoIterator<Item = S>, F: Fn(&mut P, &[String]) + 'static>(&mut self, aliases: I, argct: usize, f: F) {
        let id = Rc::new(());
        let f = Rc::new(f);
        for alias in aliases.into_iter() {
            self.0.match_n(alias.as_ref(), id.clone(), argct, f.clone());
        }
    }

    pub fn match_extra_soft<F: Fn(&mut P, &str) -> bool + 'static>(&mut self, f: F) {
        self.0.match_extra_soft(Rc::new(f));
    }

    pub fn match_extra_hard<F: Fn(&mut P, &[String]) + 'static>(&mut self, f: F) {
        self.0.match_extra_hard(Rc::new(f));
    }
}



pub struct OptParser<P> {
    named: NameTrie<(String, Rc<()>, usize, Rc<Fn(&mut P, &[String])>)>,
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
            named: NameTrie::new(),
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
                    let (_, _, argct, f) = self.named.get(name).iter().fold(None, |hit, (name2, id2, argct2, f2)| {
                        if let Some((name1, id1, _, _)) = hit {
                            if !Rc::ptr_eq(id1, id2) {
                                panic!("Option {} is ambiguous (e.g.  {} and {})", name, name1, name2);
                            }
                        }
                        return Some((name2, id2, argct2, f2));
                    }).expect(&format!("No such option {}", name));
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
    pub fn parse(&self, args: &Vec<String>) -> P {
        let mut p = P::default();
        self.parse_mut(args, &mut p);
        return p;
    }
}

impl<'a, P: 'static> OptParserMatch<P> for &'a mut OptParser<P> {
    fn match_n(&mut self, alias: &str, id: Rc<()>, argct: usize, f: Rc<Fn(&mut P, &[String])>) {
        self.named.insert(alias, (alias.to_string(), id, argct, f.clone()));
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
    fn match_n(&mut self, alias: &str, id: Rc<()>, argct: usize, f: Rc<Fn(&mut P, &[String])>) {
        let f1 = self.f.clone();
        self.parent.match_n(alias, id, argct, Rc::new(move |p, a| f(f1(p), a)));
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
