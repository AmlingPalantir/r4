use std::sync::Arc;
use validates::Validates;
use validates::ValidationError;
use validates::ValidationResult;

#[derive(Default)]
pub struct BooleanOption(bool);

impl Validates for BooleanOption {
    type Target = bool;

    fn validate(self) -> ValidationResult<bool> {
        return Result::Ok(self.0);
    }
}

impl BooleanOption {
    pub fn set(&mut self) -> ValidationResult<()> {
        self.0 = true;
        return Result::Ok(());
    }

    pub fn clear(&mut self) -> ValidationResult<()> {
        self.0 = false;
        return Result::Ok(());
    }
}

pub trait OptionDefaulter<T> {
    fn default() -> T;
}

#[macro_export]
macro_rules! option_defaulters {
    {$($id:ident: $r:ty => $e:expr,)*} => {
        $(
            // arggh, can't derive Default on stuff templated on this
            // otherwise...
            #[derive(Default)]
            pub struct $id();

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

    fn validate(self) -> ValidationResult<T> {
        if let Some(t) = self.0 {
            return Result::Ok(t);
        }
        return Result::Ok(P::default());
    }
}

impl<T, P> DefaultedOption<T, P> {
    pub fn set(&mut self, t: T) -> ValidationResult<()> {
        if self.0.is_some() {
            return ValidationError::message("DefaultedOption specified multiple times".to_string());
        }
        self.0 = Some(t);
        return Result::Ok(());
    }

    pub fn maybe_set(&mut self, t: T) -> ValidationResult<bool> {
        if self.0.is_some() {
            return Result::Ok(false);
        }
        self.0 = Some(t);
        return Result::Ok(true);
    }

    pub fn maybe_set_with<F: FnOnce() -> T>(&mut self, f: F) -> ValidationResult<bool> {
        if self.0.is_some() {
            return Result::Ok(false);
        }
        self.0 = Some(f());
        return Result::Ok(true);
    }
}

impl<T> OptionDefaulter<T> for PanicDefaulter {
    fn default() -> T {
        panic!("Missing option");
    }
}

pub type DefaultedStringOption<P> = DefaultedOption<String, P>;

impl<P> DefaultedStringOption<P> {
    pub fn set_str(&mut self, a: &str) -> ValidationResult<()> {
        return self.set(a.to_string());
    }

    pub fn maybe_set_str(&mut self, a: &str) -> ValidationResult<bool> {
        return self.maybe_set(a.to_string());
    }
}

pub enum PanicDefaulter {
}

pub type RequiredOption<T> = DefaultedOption<T, PanicDefaulter>;

pub type RequiredStringOption = DefaultedStringOption<PanicDefaulter>;

pub type OptionalOption<T> = UnvalidatedOption<Option<T>>;

impl<T> OptionalOption<T> {
    pub fn set(&mut self, t: T) -> ValidationResult<()> {
        if self.0.is_some() {
            return ValidationError::message("OptionalOption specified multiple times".to_string());
        }
        self.0 = Some(t);
        return Result::Ok(());
    }
}

pub type OptionalStringOption = OptionalOption<String>;

impl OptionalStringOption {
    pub fn set_str(&mut self, a: &str) -> ValidationResult<()> {
        return self.set(a.to_string());
    }
}

#[derive(Default)]
pub struct UnvalidatedOption<T>(pub T);

impl<T> Validates for UnvalidatedOption<T> {
    type Target = T;

    fn validate(self) -> ValidationResult<T> {
        return Result::Ok(self.0);
    }
}

pub type StringVecOption = UnvalidatedOption<Vec<String>>;

impl StringVecOption {
    pub fn push(&mut self, s: &str) -> ValidationResult<()> {
        self.0.push(s.to_string());
        return Result::Ok(());
    }

    pub fn push_split(&mut self, s: &str) -> ValidationResult<()> {
        for a in s.split(',') {
            self.push(a)?;
        }
        return Result::Ok(());
    }

    pub fn push_all(&mut self, a: &[String]) -> ValidationResult<()> {
        for a in a {
            self.0.push(a.clone());
        }
        return Result::Ok(());
    }

    pub fn maybe_push(&mut self, a: &str) -> ValidationResult<bool> {
        return self.push(a).map(|_| true);
    }
}

pub type OptionalUsizeOption = OptionalOption<usize>;

impl OptionalUsizeOption {
    pub fn parse(&mut self, a: &str) -> ValidationResult<()> {
        return self.set(a.parse().unwrap());
    }
}

#[derive(Default)]
pub struct IntoArcOption<P>(pub P);

impl<P: Validates> Validates for IntoArcOption<P> {
    type Target = Arc<P::Target>;

    fn validate(self) -> ValidationResult<Arc<P::Target>> {
        return self.0.validate().map(Arc::new);
    }
}

#[derive(Default)]
pub struct EmptyOption();

impl Validates for EmptyOption {
    type Target = ();

    fn validate(self) -> ValidationResult<()> {
        return Result::Ok(());
    }
}
