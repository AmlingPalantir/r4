#[macro_export]
macro_rules! declare_opts {
    {$($name:ident: $type:ty,)*} => {
        #[derive(Default)]
        pub struct PreOptions {
            $(
                $name: $type,
            )*
        }

        impl $crate::vals::OptionTrait for PreOptions {
            type ValidatesTo = PostOptions;

            fn validate(self) -> PostOptions {
                return PostOptions {
                    $(
                        $name: <$type as $crate::vals::OptionTrait>::validate(self.$name),
                    )*
                };
            }
        }

        #[derive(Clone)]
        pub struct PostOptions {
            $(
                $name: <$type as $crate::vals::OptionTrait>::ValidatesTo,
            )*
        }
    }
}

pub trait OptionTrait {
    type ValidatesTo;

    fn validate(self) -> Self::ValidatesTo;
}

#[derive(Default)]
pub struct BooleanOption(bool);

impl OptionTrait for BooleanOption {
    type ValidatesTo = bool;

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

impl<T> OptionTrait for RequiredOption<T> {
    type ValidatesTo = T;

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

pub type RequiredStringOption = RequiredOption<String>;

impl RequiredStringOption {
    pub fn set_str(&mut self, a: &str) {
        self.set(a.to_string());
    }
}

pub struct OptionalOption<T>(Option<T>);

impl<T> Default for OptionalOption<T> {
    fn default() -> Self {
        return OptionalOption(None);
    }
}

impl<T> OptionTrait for OptionalOption<T> {
    type ValidatesTo = Option<T>;

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

#[derive(Default)]
pub struct UnvalidatedOption<T>(T);

impl<T> AsMut<T> for UnvalidatedOption<T> {
    fn as_mut(&mut self) -> &mut T {
        return &mut self.0;
    }
}

impl<T> OptionTrait for UnvalidatedOption<T> {
    type ValidatesTo = T;

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
}
