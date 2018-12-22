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
    pub fn parse(&self, args: &mut Vec<String>) {
        let mut p = P::default();

        let mut save_index = 0;
        let mut next_index = 0;
        let mut refuse_opt = false;
        'arg: loop {
            if next_index == args.len() {
                args.truncate(save_index);
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

            args.swap(save_index, next_index);
            save_index += 1;
            next_index += 1;
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

    pub fn match_single<F: Fn(&mut P, &String) + 'static>(&mut self, aliases: &[String], f: F) {
        self.match_n(aliases, 1, move |p, a| f(p, &a[0]));
    }

    pub fn match_zero<F: Fn(&mut P) + 'static>(&mut self, aliases: &[String], f: F) {
        self.match_n(aliases, 0, move |p, _a| f(p));
    }

    pub fn match_n<F: Fn(&mut P, &[String]) + 'static>(&mut self, aliases: &[String], argct: usize, f: F) {
        let f = Rc::new(f);
        for alias in aliases {
            let prev = self.op.named.insert(alias.clone(), (argct, f.clone()));
            if prev.is_some() {
                panic!();
            }
        }
    }

    pub fn match_extra_soft<F: Fn(&mut P, &String) -> bool + 'static>(&mut self, f: F) {
        self.op.extra.push(ExtraHandler::Soft(Rc::new(f)));
    }

    pub fn match_extra_hard<F: Fn(&mut P, &[String]) + 'static>(&mut self, f: F) {
        self.op.extra.push(ExtraHandler::Hard(Rc::new(f)));
    }
}
