extern crate opts;
extern crate validates;

pub mod args;
use self::args::RegistryArgs;

use opts::parser::OptParserView;
use std::collections::HashMap;
use validates::ValidationError;
use validates::ValidationResult;

pub struct Registry<R> {
    map: HashMap<String, (usize, Box<Fn(&[&str]) -> ValidationResult<R> + Send + Sync>)>,
}

impl<R> Default for Registry<R> {
    fn default() -> Self {
        return Registry {
            map: HashMap::new(),
        };
    }
}

impl<R> Registry<R> {
    pub fn add<F: Fn(&[&str]) -> ValidationResult<R> + Send + Sync + 'static>(&mut self, name: &str, argct: usize, f: F) {
        let prev = self.map.insert(name.to_string(), (argct, Box::new(f)));
        assert!(prev.is_none(), "registry collision for {}", name);
    }

    pub fn find(&self, name: &str, args: &[&str]) -> ValidationResult<R> {
        match self.map.get(name) {
            None => {
                return ValidationError::message(format!("No implementation named {}", name));
            }
            Some((argct, f)) => {
                if args.len() != *argct {
                    return ValidationError::message(format!("Wrong number of args for {}", name));
                }
                return f(args);
            }
        }
    }

    pub fn labelled_multiple_options<'a, O: AsMut<Vec<(String, R)>> + 'static>(&'static self, opt: &mut OptParserView<'a, O>, prefixes: &[&str]) {
        for (alias, (argct, f)) in &self.map {
            let aliases: Vec<_> = prefixes.iter().map(|prefix| format!("{}-{}", prefix, alias)).collect();
            let aliases: Vec<_> = aliases.iter().map(|s| s as &str).collect();
            opt.match_n(&aliases, argct + 1, move |rs, a| {
                let mut iter = a.iter();
                let label = iter.next().unwrap().to_string();
                let a: Vec<_> = iter.map(|s| s as &str).collect();
                rs.as_mut().push((label, f(&a)?));
                return Result::Ok(());
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
            let r = self.find(name, &args)?;
            rs.as_mut().push((label, r));
            return Result::Ok(());
        });
    }

    pub fn multiple_options<'a, O: AsMut<Vec<R>> + 'static>(&'static self, opt: &mut OptParserView<'a, O>, prefixes: &[&str]) {
        for (alias, (argct, f)) in &self.map {
            let aliases: Vec<_> = prefixes.iter().map(|prefix| format!("{}-{}", prefix, alias)).collect();
            let aliases: Vec<_> = aliases.iter().map(|s| s as &str).collect();
            opt.match_n(&aliases, *argct, move |rs, a| {
                let a: Vec<_> = a.iter().map(|s| s as &str).collect();
                rs.as_mut().push(f(&a)?);
                return Result::Ok(());
            });
        }
    }

    pub fn single_options<'a, O: AsMut<Vec<R>> + 'static>(&'static self, opt: &mut OptParserView<'a, O>, aliases: &[&str]) {
        opt.match_single(aliases, move |rs, a| {
            let mut parts = a.split(',');
            let name = parts.next().unwrap();
            let args: Vec<_> = parts.collect();
            let r = self.find(name, &args)?;
            rs.as_mut().push(r);
            return Result::Ok(());
        });
    }
}

#[macro_export]
macro_rules! registry {
    {$r:ty, $($id:ident,)*} => {
        $(
            pub mod $id;
        )*

        lazy_static! {
            pub static ref REGISTRY: $crate::Registry<$r> = {
                let mut r = $crate::Registry::default();
                $(
                    for name in <$id::Impl as $crate::Registrant<$r>>::names() {
                        r.add(name, <$id::Impl as $crate::Registrant<$r>>::argct(), <$id::Impl as $crate::Registrant<$r>>::init);
                    }
                )*
                r
            };
        }
    }
}

pub trait Registrant<R> {
    type Args: RegistryArgs;

    fn names() -> Vec<&'static str>;
    fn init2(a: <Self::Args as RegistryArgs>::Val) -> R;

    fn argct() -> usize {
        return Self::Args::argct();
    }

    fn init(args: &[&str]) -> ValidationResult<R> {
        return Result::Ok(Self::init2(Self::Args::parse(args)?));
    }
}
