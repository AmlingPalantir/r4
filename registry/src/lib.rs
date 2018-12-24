use std::sync::Arc;

#[macro_export]
macro_rules! registry {
    {$r:ty: $($id:ident,)*} => {
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
            panic!("No implementation named {}", name);
        }
    }
}



pub trait RegistryArgs {
    type Val: Send + Sync;

    fn argct() -> usize;
    fn parse(args: &[&str]) -> Self::Val;
}



pub enum ZeroArgs {
}

impl RegistryArgs for ZeroArgs {
    type Val = ();

    fn argct() -> usize {
        return 0;
    }

    fn parse(args: &[&str]) -> () {
        assert_eq!(0, args.len());
        return ();
    }
}



pub enum OneStringArgs {
}

impl RegistryArgs for OneStringArgs {
    type Val = Arc<str>;

    fn argct() -> usize {
        return 1;
    }

    fn parse(args: &[&str]) -> Arc<str> {
        assert_eq!(1, args.len());
        return Arc::from(&*args[0]);
    }
}
