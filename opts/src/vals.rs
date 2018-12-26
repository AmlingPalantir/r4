use std::collections::HashSet;
use std::sync::Arc;
use validates::Validates;

#[macro_export]
macro_rules! declare_opts {
    {$($name:ident: $type:ty,)*} => {
        #[derive(Default)]
        pub struct PreOptions {
            $(
                $name: $type,
            )*
        }

        impl validates::Validates for PreOptions {
            type Target = PostOptions;

            fn validate(self) -> PostOptions {
                return PostOptions {
                    $(
                        $name: <$type as validates::Validates>::validate(self.$name),
                    )*
                };
            }
        }

        #[derive(Clone)]
        pub struct PostOptions {
            $(
                $name: <$type as validates::Validates>::Target,
            )*
        }
    }
}

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

pub struct RequiredOption<T>(Option<T>);

impl<T> Default for RequiredOption<T> {
    fn default() -> Self {
        return RequiredOption(None);
    }
}

impl<T> Validates for RequiredOption<T> {
    type Target = T;

    fn validate(self) -> T {
        return self.0.unwrap();
    }
}

impl<T> RequiredOption<T> {
    pub fn set(&mut self, t: T) {
        if let Some(_) = self.0 {
            panic!("RequiredOption specified multiple times");
        }
        self.0 = Some(t);
    }
}

impl<T: Clone> RequiredOption<T> {
    pub fn set_clone(&mut self, t: &T) {
        self.set(t.clone());
    }
}

#[derive(Default)]
pub struct RequiredStringOption(Option<String>);

impl Validates for RequiredStringOption {
    type Target = Arc<str>;

    fn validate(self) -> Arc<str> {
        return match self.0 {
            Some(s) => Arc::from(&s as &str),
            None => panic!("RequiredStringOption not specified"),
        };
    }
}

impl RequiredStringOption {
    pub fn set(&mut self, a: &str) {
        if let Some(_) = self.0 {
            panic!("RequiredStringOption specified multiple times");
        }
        self.0 = Some(a.to_string());
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

#[derive(Default)]
pub struct OptionalStringOption(Option<String>);

impl Validates for OptionalStringOption {
    type Target = Option<Arc<str>>;

    fn validate(self) -> Option<Arc<str>> {
        return match self.0 {
            Some(s) => Some(Arc::from(&s as &str)),
            None => None,
        };
    }
}

impl OptionalStringOption {
    pub fn set(&mut self, a: &str) {
        if let Some(_) = self.0 {
            panic!("OptionalStringOption specified multiple times");
        }
        self.0 = Some(a.to_string());
    }
}

#[derive(Default)]
pub struct UnvalidatedArcOption<T>(pub T);

impl<T> Validates for UnvalidatedArcOption<T> {
    type Target = Arc<T>;

    fn validate(self) -> Arc<T> {
        return Arc::new(self.0);
    }
}

pub type StringVecOption = UnvalidatedArcOption<Vec<String>>;

impl StringVecOption {
    pub fn push(&mut self, s: &str) {
        self.0.push(s.to_string());
    }

    pub fn push_split(&mut self, s: &str) {
        for a in s.split(',') {
            self.push(a);
        }
    }
}

#[derive(Default)]
pub struct StringSetOption(Vec<String>);

impl Validates for StringSetOption {
    type Target = Arc<HashSet<String>>;

    fn validate(self) -> Arc<HashSet<String>> {
        return Arc::new(self.0.into_iter().collect());
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
