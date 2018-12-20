#[macro_export]
macro_rules! registry {
    {$r:ty: $($id:ident),*} => {
        $(
            pub mod $id;
        )*

        pub fn find(name: &str) -> Box<$r> {
            $(
                for name2 in $id::names() {
                    if name == name2 {
                        return Box::new($id::Impl::default());
                    }
                }
            )*
            panic!();
        }
    };
    {$r:ty: $($id:ident),*,} => {
        registry! {$r: $($id),*}
    };
}
