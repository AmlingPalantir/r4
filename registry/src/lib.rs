extern crate opts;

use opts::parser::OptParserView;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

pub struct Registry<R> {
    map: HashMap<String, (usize, Box<Fn(&[&str]) -> R + Send + Sync>)>,
}

impl<R> Default for Registry<R> {
    fn default() -> Self {
        return Registry {
            map: HashMap::new(),
        };
    }
}

impl<R> Registry<R> {
    pub fn add<F: Fn(&[&str]) -> R + Send + Sync + 'static>(&mut self, name: &str, argct: usize, f: F) {
        let prev = self.map.insert(name.to_string(), (argct, Box::new(f)));
        assert!(prev.is_none(), "registry collision for {}", name);
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

    pub fn labelled_multiple_options<'a, O: AsMut<Vec<(String, R)>> + 'static>(&'static self, opt: &mut OptParserView<'a, O>, prefixes: &[&str]) {
        for (alias, (argct, f)) in &self.map {
            let aliases: Vec<_> = prefixes.iter().map(|prefix| format!("{}-{}", prefix, alias)).collect();
            opt.match_n(aliases, argct + 1, move |rs, a| {
                let mut iter = a.iter();
                let label = iter.next().unwrap().to_string();
                let a: Vec<_> = iter.map(|s| s as &str).collect();
                rs.as_mut().push((label, f(&a)));
            });
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

    pub fn multiple_options<'a, O: AsMut<Vec<R>> + 'static>(&'static self, opt: &mut OptParserView<'a, O>, prefixes: &[&str]) {
        for (alias, (argct, f)) in &self.map {
            let aliases: Vec<_> = prefixes.iter().map(|prefix| format!("{}-{}", prefix, alias)).collect();
            opt.match_n(aliases, *argct, move |rs, a| {
                let a: Vec<_> = a.iter().map(|s| s as &str).collect();
                rs.as_mut().push(f(&a));
            });
        }
    }

    pub fn single_options<'a, O: AsMut<Vec<R>> + 'static>(&'static self, opt: &mut OptParserView<'a, O>, aliases: &[&str]) {
        opt.match_single(aliases, move |rs, a| {
            let mut parts = a.split(',');
            let name = parts.next().unwrap();
            let args: Vec<_> = parts.collect();
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
                let mut r = $crate::Registry::default();
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



pub struct OneFromStrArgs<T: FromStr> {
    _x: std::marker::PhantomData<T>,
}

impl<T: FromStr + Send + Sync> RegistryArgs for OneFromStrArgs<T> where T::Err: Debug {
    type Val = T;

    fn argct() -> usize {
        return 1;
    }

    fn parse(args: &[&str]) -> T {
        assert_eq!(1, args.len());
        return T::from_str(args[0]).unwrap();
    }
}



pub type OneIntArgs = OneFromStrArgs<i64>;
pub type OneUsizeArgs = OneFromStrArgs<usize>;



pub enum TwoStringArgs {
}

impl RegistryArgs for TwoStringArgs {
    type Val = (Arc<str>, Arc<str>);

    fn argct() -> usize {
        return 2;
    }

    fn parse(args: &[&str]) -> (Arc<str>, Arc<str>) {
        assert_eq!(2, args.len());
        return (Arc::from(&*args[0]), Arc::from(&*args[1]));
    }
}



pub enum ThreeStringArgs {
}

impl RegistryArgs for ThreeStringArgs {
    type Val = (Arc<str>, Arc<str>, Arc<str>);

    fn argct() -> usize {
        return 3;
    }

    fn parse(args: &[&str]) -> (Arc<str>, Arc<str>, Arc<str>) {
        assert_eq!(3, args.len());
        return (Arc::from(&*args[0]), Arc::from(&*args[1]), Arc::from(&*args[2]));
    }
}
