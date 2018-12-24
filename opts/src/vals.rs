use std::ops::Deref;
use std::ops::DerefMut;

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

#[derive(Default)]
pub struct RequiredStringOption(Option<String>);

impl OptionTrait for RequiredStringOption {
    type ValidatesTo = String;

    fn validate(self) -> String {
        return self.0.unwrap();
    }
}

impl RequiredStringOption {
    pub fn set(&mut self, a: &String) {
        if let Some(_) = self.0 {
            panic!("RequiredStringOption missing");
        }
        self.0 = Some(a.clone());
    }
}

#[derive(Default)]
pub struct UnvalidatedOption<T>(T);

impl<T> Deref for UnvalidatedOption<T> {
    type Target = T;

    fn deref(&self) -> &T {
        return &self.0;
    }
}

impl<T> DerefMut for UnvalidatedOption<T> {
    fn deref_mut(&mut self) -> &mut T {
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
