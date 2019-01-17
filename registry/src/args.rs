use std::str::FromStr;
use std::sync::Arc;
use validates::ValidationResult;

pub trait RegistryArgs {
    type Val: Send + Sync;

    fn argct() -> usize;
    fn parse(args: &[&str]) -> ValidationResult<Self::Val>;
}

pub enum ZeroArgs {
}

impl RegistryArgs for ZeroArgs {
    type Val = ();

    fn argct() -> usize {
        return 0;
    }

    fn parse(args: &[&str]) -> ValidationResult<()> {
        assert_eq!(0, args.len());
        return Result::Ok(());
    }
}

pub enum OneStringArgs {
}

impl RegistryArgs for OneStringArgs {
    type Val = Arc<str>;

    fn argct() -> usize {
        return 1;
    }

    fn parse(args: &[&str]) -> ValidationResult<Arc<str>> {
        assert_eq!(1, args.len());
        return Result::Ok(Arc::from(&*args[0]));
    }
}

pub struct OneFromStrArgs<T: FromStr> {
    _x: std::marker::PhantomData<T>,
}

impl<T: FromStr + Send + Sync> RegistryArgs for OneFromStrArgs<T> where T::Err: std::error::Error + 'static {
    type Val = T;

    fn argct() -> usize {
        return 1;
    }

    fn parse(args: &[&str]) -> ValidationResult<T> {
        assert_eq!(1, args.len());
        return Result::Ok(T::from_str(args[0])?);
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

    fn parse(args: &[&str]) -> ValidationResult<(Arc<str>, Arc<str>)> {
        assert_eq!(2, args.len());
        return Result::Ok((Arc::from(&*args[0]), Arc::from(&*args[1])));
    }
}

pub enum ThreeStringArgs {
}

impl RegistryArgs for ThreeStringArgs {
    type Val = (Arc<str>, Arc<str>, Arc<str>);

    fn argct() -> usize {
        return 3;
    }

    fn parse(args: &[&str]) -> ValidationResult<(Arc<str>, Arc<str>, Arc<str>)> {
        assert_eq!(3, args.len());
        return Result::Ok((Arc::from(&*args[0]), Arc::from(&*args[1]), Arc::from(&*args[2])));
    }
}
