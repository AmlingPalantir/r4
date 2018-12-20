extern crate record;
extern crate stream;

macro_rules! registry {
    {$($id:ident),*} => {
        $(
            pub mod $id;
        )*

        pub fn find_operation(name: &str) -> Box<Operation> {
            $(
                if name == $id::name() {
                    return $id::new();
                }
            )*
            panic!();
        }
    };
    {$($id:ident),*,} => {
        registry! {$($id),*}
    };
}

registry! {
    test,
}

use stream::Stream;

pub trait Operation {
    fn configure(&mut self, Vec<String>) -> Vec<String>;
    fn validate(&self) -> Box<StreamWrapper>;
}

pub trait StreamWrapper {
    fn wrap(&self, Box<Stream>) -> Box<Stream>;
}
