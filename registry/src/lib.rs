#[macro_export]
macro_rules! registry {
    {$r:ty: $($id:ident),*} => {
        $(
            pub mod $id;
        )*

        pub fn find(name: &str) -> Box<$r> {
            $(
                if name == $id::name() {
                    return Box::new($id::new());
                }
            )*
            panic!();
        }
    };
    {$r:ty: $($id:ident),*,} => {
        registry! {$r: $($id),*}
    };
}
