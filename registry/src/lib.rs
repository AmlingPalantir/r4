use std::sync::Arc;

#[macro_export]
macro_rules! registry {
    {$fe:ident, $r:ty, $($id:ident,)*} => {
        $(
            mod $id;
        )*

        pub fn find(name: &str, args: &[&str]) -> $r {
            $(
                for name2 in <$id::Impl as $fe>::names() {
                    if name == name2 {
                        if args.len() != <$id::Impl as $fe>::argct() {
                            panic!("Wrong number of args for {}", name);
                        }
                        return <$id::Impl as $fe>::init(args);
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
