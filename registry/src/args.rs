use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

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

    fn parse(args: &[&str]) {
        assert_eq!(0, args.len());
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

pub struct OneFromStrArgs<T: FromStr> {
    _x: std::marker::PhantomData<T>,
}

impl<T: FromStr + Send + Sync> RegistryArgs for OneFromStrArgs<T> where T::Err: Debug {
    type Val = T;

    fn argct() -> usize {
        return 1;
    }

    fn parse(args: &[&str]) -> T {
        assert_eq!(1, args.len());
        return T::from_str(args[0]).unwrap();
    }
}

pub type OneIntArgs = OneFromStrArgs<i64>;
pub type OneUsizeArgs = OneFromStrArgs<usize>;

pub enum TwoStringArgs {
}

impl RegistryArgs for TwoStringArgs {
    type Val = (Arc<str>, Arc<str>);

    fn argct() -> usize {
        return 2;
    }

    fn parse(args: &[&str]) -> (Arc<str>, Arc<str>) {
        assert_eq!(2, args.len());
        return (Arc::from(&*args[0]), Arc::from(&*args[1]));
    }
}

pub enum ThreeStringArgs {
}

impl RegistryArgs for ThreeStringArgs {
    type Val = (Arc<str>, Arc<str>, Arc<str>);

    fn argct() -> usize {
        return 3;
    }

    fn parse(args: &[&str]) -> (Arc<str>, Arc<str>, Arc<str>) {
        assert_eq!(3, args.len());
        return (Arc::from(&*args[0]), Arc::from(&*args[1]), Arc::from(&*args[2]));
    }
}
