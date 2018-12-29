use std::collections::HashSet;
use std::sync::Arc;
use validates::Validates;

#[derive(Default)]
pub struct BooleanOption(bool);

impl Validates for BooleanOption {
    type Target = bool;

    fn validate(self) -> bool {
        return self.0;
    }
}

impl BooleanOption {
    pub fn set(&mut self) {
        self.0 = true;
    }

    pub fn clear(&mut self) {
        self.0 = false;
    }
}

pub trait OptionDefaulter<T> {
    fn default() -> T;
}

#[macro_export]
macro_rules! option_defaulters {
    {$($id:ident: $r:ty => $e:expr,)*} => {
        $(
            enum $id {
            }

            impl $crate::vals::OptionDefaulter<$r> for $id {
                fn default() -> $r {
                    return $e;
                }
            }
        )*
    }
}

pub struct DefaultedOption<T, P>(Option<T>, std::marker::PhantomData<P>);

impl<T, P> Default for DefaultedOption<T, P> {
    fn default() -> Self {
        return DefaultedOption(None, std::marker::PhantomData::default());
    }
}

impl<T, P: OptionDefaulter<T>> Validates for DefaultedOption<T, P> {
    type Target = T;

    fn validate(self) -> T {
        if let Some(t) = self.0 {
            return t;
        }
        return P::default();
    }
}

impl<T, P> DefaultedOption<T, P> {
    pub fn set(&mut self, t: T) {
        if let Some(_) = self.0 {
            panic!("DefaultedOption specified multiple times");
        }
        self.0 = Some(t);
    }

    pub fn maybe_set(&mut self, t: T) -> bool {
        if let Some(_) = self.0 {
            return false;
        }
        self.0 = Some(t);
        return true;
    }

    pub fn maybe_set_with<F: FnOnce() -> T>(&mut self, f: F) -> bool {
        if let Some(_) = self.0 {
            return false;
        }
        self.0 = Some(f());
        return true;
    }
}

impl<T: Clone, P> DefaultedOption<T, P> {
    pub fn set_clone(&mut self, t: &T) {
        self.set(t.clone());
    }
}

pub enum PanicDefaulter {
}

impl<T> OptionDefaulter<T> for PanicDefaulter {
    fn default() -> T {
        panic!("Missing option");
    }
}

pub type RequiredOption<T> = DefaultedOption<T, PanicDefaulter>;

pub type RequiredStringOption = RequiredOption<String>;

impl RequiredStringOption {
    pub fn set_str(&mut self, a: &str) {
        self.set(a.to_string());
    }

    pub fn maybe_set_str(&mut self, a: &str) -> bool {
        return self.maybe_set(a.to_string());
    }
}

pub struct OptionalOption<T>(Option<T>);

impl<T> Default for OptionalOption<T> {
    fn default() -> Self {
        return OptionalOption(None);
    }
}

impl<T> Validates for OptionalOption<T> {
    type Target = Option<T>;

    fn validate(self) -> Option<T> {
        return self.0;
    }
}

impl<T> OptionalOption<T> {
    pub fn set(&mut self, t: T) {
        if let Some(_) = self.0 {
            panic!("OptionalOption specified multiple times");
        }
        self.0 = Some(t);
    }
}

impl<T: Clone> OptionalOption<T> {
    pub fn set_clone(&mut self, t: &T) {
        self.set(t.clone());
    }
}

pub type OptionalStringOption = OptionalOption<String>;

impl OptionalStringOption {
    pub fn set_str(&mut self, a: &str) {
        self.set(a.to_string());
    }
}

#[derive(Default)]
pub struct UnvalidatedOption<T>(pub T);

impl<T> Validates for UnvalidatedOption<T> {
    type Target = T;

    fn validate(self) -> T {
        return self.0;
    }
}

pub type StringVecOption = UnvalidatedOption<Vec<String>>;

impl StringVecOption {
    pub fn push(&mut self, s: &str) {
        self.0.push(s.to_string());
    }

    pub fn push_split(&mut self, s: &str) {
        for a in s.split(',') {
            self.push(a);
        }
    }

    pub fn push_all(&mut self, a: &[String]) {
        for a in a {
            self.0.push(a.clone());
        }
    }

    pub fn maybe_push(&mut self, a: &str) -> bool {
        self.push(a);
        return true;
    }
}

#[derive(Default)]
pub struct StringSetOption(Vec<String>);

impl Validates for StringSetOption {
    type Target = HashSet<String>;

    fn validate(self) -> HashSet<String> {
        return self.0.into_iter().collect();
    }
}

impl StringSetOption {
    pub fn push(&mut self, s: &str) {
        self.0.push(s.to_string());
    }

    pub fn push_split(&mut self, s: &str) {
        for a in s.split(',') {
            self.push(a);
        }
    }
}

pub type OptionalUsizeOption = OptionalOption<usize>;

impl OptionalUsizeOption {
    pub fn parse(&mut self, a: &str) {
        self.set(a.parse().unwrap());
    }
}

#[derive(Clone)]
#[derive(Default)]
pub struct IntoArcOption<P>(pub P);

impl<P: Validates> Validates for IntoArcOption<P> {
    type Target = Arc<<P as Validates>::Target>;

    fn validate(self) -> Arc<<P as Validates>::Target> {
        return Arc::new(self.0.validate());
    }
}
