extern crate opts;
extern crate validates;

pub mod args;
use self::args::RegistryArgs;

use opts::parser::OptionsPile;
use opts::parser::ToOptionsHelp;
use std::collections::HashMap;
use std::sync::Arc;
use validates::ValidationError;
use validates::ValidationResult;

struct RegistrantData<R> {
    names: Vec<&'static str>,
    argct: usize,
    init: Box<Fn(&[&str]) -> ValidationResult<R> + Send + Sync>,
}

pub struct Registry<R> {
    map: HashMap<&'static str, Arc<RegistrantData<R>>>,
    list: Vec<Arc<RegistrantData<R>>>,
}

impl<R> Default for Registry<R> {
    fn default() -> Self {
        return Registry {
            map: HashMap::new(),
            list: Vec::new(),
        };
    }
}

impl<R: 'static> Registry<R> {
    pub fn add<I: Registrant<R> + 'static>(&mut self) {
        let data = Arc::new(RegistrantData {
            names: I::names(),
            argct: I::argct(),
            init: Box::new(I::init),
        });
        for name in &data.names {
            let prev = self.map.insert(name, data.clone());
            assert!(prev.is_none(), "registry collision for {}", name);
        }
        self.list.push(data);
    }

    fn find_data(&self, name: &str) -> ValidationResult<Arc<RegistrantData<R>>> {
        return match self.map.get(name) {
            None => ValidationError::message(format!("No implementation named {}", name)),
            Some(data) => Result::Ok(data.clone()),
        };
    }

    pub fn find(&self, name: &str, args: &[&str]) -> ValidationResult<R> {
        let data = self.find_data(name)?;

        if args.len() != data.argct {
            return ValidationError::message(format!("Wrong number of args for {}", name));
        }

        return (data.init)(args);
    }

    pub fn labelled_multiple_options(&'static self, prefixes: &[&str]) -> OptionsPile<Vec<(String, R)>> {
        let mut opt = OptionsPile::<Vec<(String, R)>>::new();
        for (alias, data) in &self.map {
            let aliases: Vec<_> = prefixes.iter().map(|prefix| format!("{}-{}", prefix, alias)).collect();
            let aliases: Vec<_> = aliases.iter().map(|s| s as &str).collect();
            opt.match_n(&aliases, data.argct + 1, move |rs, a| {
                let mut iter = a.iter();
                let label = iter.next().unwrap().to_string();
                let a: Vec<_> = iter.map(|s| s as &str).collect();
                rs.push((label, (data.init)(&a)?));
                return Result::Ok(());
            }, None);
        }
        return opt;
    }

    pub fn labelled_single_options(&'static self, aliases: &[&str], help: impl ToOptionsHelp) -> OptionsPile<Vec<(String, R)>> {
        let mut opt = OptionsPile::<Vec<(String, R)>>::new();
        opt.match_single(aliases, move |rs, a| {
            let (label, a) = match a.find('=') {
                Some(i) => (a[0..i].to_string(), &a[(i + 1)..]),
                None => (a.replace("/", "_"), &a[..]),
            };
            let mut parts = a.split(',');
            let name = parts.next().unwrap();
            let args: Vec<&str> = parts.collect();
            let r = self.find(name, &args)?;
            rs.push((label, r));
            return Result::Ok(());
        }, help);
        return opt;
    }

    pub fn multiple_options(&'static self, prefixes: &[&str]) -> OptionsPile<Vec<R>> {
        let mut opt = OptionsPile::<Vec<R>>::new();
        for (alias, data) in &self.map {
            let aliases: Vec<_> = prefixes.iter().map(|prefix| format!("{}-{}", prefix, alias)).collect();
            let aliases: Vec<_> = aliases.iter().map(|s| s as &str).collect();
            opt.match_n(&aliases, data.argct, move |rs, a| {
                let a: Vec<_> = a.iter().map(|s| s as &str).collect();
                rs.push((data.init)(&a)?);
                return Result::Ok(());
            }, None);
        }
        return opt;
    }

    pub fn single_options(&'static self, aliases: &[&str], help: impl ToOptionsHelp) -> OptionsPile<Vec<R>> {
        let mut opt = OptionsPile::<Vec<R>>::new();
        opt.match_single(aliases, move |rs, a| {
            let mut parts = a.split(',');
            let name = parts.next().unwrap();
            let args: Vec<_> = parts.collect();
            let r = self.find(name, &args)?;
            rs.push(r);
            return Result::Ok(());
        }, help);
        return opt;
    }

    pub fn help_options<X: 'static>(&'static self, type_name: &str) -> OptionsPile<X> {
        let mut opt = OptionsPile::<X>::new();
        let list = &self.list;
        opt.match_zero(&[&format!("list-{}", type_name)], move |_p| {
            return ValidationError::help(list.iter().map(|data| {
                let (first, rest) = data.names.split_first().unwrap();
                let mut line = first.to_string();
                if !rest.is_empty() {
                    line.push_str(&format!(" [{}]", rest.join(", ")));
                }
                return line;
            }).collect());
        }, format!("list {}s", type_name));
        return opt;
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
                    r.add::<$id::Impl>();
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
