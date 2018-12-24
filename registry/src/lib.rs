extern crate opts;

use opts::parser::OptParserView;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Registry<R> {
    map: HashMap<String, (usize, Box<Fn(&[&str]) -> R + Send + Sync>)>,
}

impl<R> Registry<R> {
    pub fn new() -> Self {
        return Registry {
            map: HashMap::new(),
        };
    }

    pub fn add<F: Fn(&[&str]) -> R + Send + Sync + 'static>(&mut self, name: &str, argct: usize, f: F) {
        let prev = self.map.insert(name.to_string(), (argct, Box::new(f)));
        assert!(prev.is_none());
    }

    pub fn find(&self, name: &str, args: &[&str]) -> R {
        match self.map.get(name) {
            None => {
                panic!("No implementation named {}", name);
            }
            Some((argct, f)) => {
                if args.len() != *argct {
                    panic!("Wrong number of args for {}", name);
                }
                return f(args);
            }
        }
    }

    pub fn labelled_single_options<'a, O: AsMut<Vec<(String, R)>> + 'static>(&'static self, opt: &mut OptParserView<'a, O>, aliases: &[&str]) {
        opt.match_single(aliases, move |rs, a| {
            let (label, a) = match a.find('=') {
                Some(i) => (a[0..i].to_string(), &a[(i + 1)..]),
                None => (a.replace("/", "_"), &a[..]),
            };
            let mut parts = a.split(',');
            let name = parts.next().unwrap();
            let args: Vec<&str> = parts.collect();
            let r = self.find(name, &args);
            rs.as_mut().push((label, r));
        });
    }

    pub fn single_options<'a, O: AsMut<Vec<R>> + 'static>(&'static self, opt: &mut OptParserView<'a, O>, aliases: &[&str]) {
        opt.match_single(aliases, move |rs, a| {
            let mut parts = a.split(',');
            let name = parts.next().unwrap();
            let args: Vec<&str> = parts.collect();
            let r = self.find(name, &args);
            rs.as_mut().push(r);
        });
    }
}

#[macro_export]
macro_rules! registry {
    {$fe:ident, $r:ty, $($id:ident,)*} => {
        $(
            pub mod $id;
        )*

        lazy_static! {
            pub static ref REGISTRY: $crate::Registry<$r> = {
                let mut r = $crate::Registry::new();
                $(
                    for name in <$id::Impl as $fe>::names() {
                        r.add(name, <$id::Impl as $fe>::argct(), <$id::Impl as $fe>::init);
                    }
                )*
                r
            };
        }
    }
}



pub trait RegistryArgs {
    type Val: Send + Sync;

    fn argct() -> usize;
    fn parse(args: &[&str]) -> Self::Val;
}



pub enum ZeroArgs {
}

impl RegistryArgs for ZeroArgs {
    type Val = ();

    fn argct() -> usize {
        return 0;
    }

    fn parse(args: &[&str]) -> () {
        assert_eq!(0, args.len());
        return ();
    }
}



pub enum OneStringArgs {
}

impl RegistryArgs for OneStringArgs {
    type Val = Arc<str>;

    fn argct() -> usize {
        return 1;
    }

    fn parse(args: &[&str]) -> Arc<str> {
        assert_eq!(1, args.len());
        return Arc::from(&*args[0]);
    }
}
