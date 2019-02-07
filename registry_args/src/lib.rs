extern crate validates;

use std::str::FromStr;
use std::sync::Arc;
use validates::ValidationResult;

pub trait RegistryArg: Send + Sized + Sync {
    fn parse(arg: &str) -> ValidationResult<Self>;
}

impl RegistryArg for Arc<str> {
    fn parse(arg: &str) -> ValidationResult<Arc<str>> {
        return Result::Ok(Arc::from(arg));
    }
}

pub trait RegistryArgs: Send + Sized + Sync {
    fn help_meta_suffix() -> &'static str;
    fn argct() -> usize;
    fn parse(args: &[&str]) -> ValidationResult<Self>;
}

pub trait MayRegistryArgFromStr {
}

impl<T: FromStr + MayRegistryArgFromStr + Send + Sync> RegistryArg for T where T::Err: std::error::Error {
    fn parse(arg: &str) -> ValidationResult<T> {
        return Result::Ok(T::from_str(arg)?);
    }
}

impl MayRegistryArgFromStr for usize {
}

//pub type OneIntArgs = OneFromStrArgs<i64>;
//pub type OneUsizeArgs = OneFromStrArgs<usize>;
//
//pub enum TwoStringArgs {
//}
//
//impl RegistryArgs for TwoStringArgs {
//    type Val = (Arc<str>, Arc<str>);
//
//    fn argct() -> usize {
//        return 2;
//    }
//
//    fn parse(args: &[&str]) -> ValidationResult<(Arc<str>, Arc<str>)> {
//        assert_eq!(2, args.len());
//        return Result::Ok((Arc::from(&*args[0]), Arc::from(&*args[1])));
//    }
//}
//
//pub enum ThreeStringArgs {
//}
//
//impl RegistryArgs for ThreeStringArgs {
//    type Val = (Arc<str>, Arc<str>, Arc<str>);
//
//    fn argct() -> usize {
//        return 3;
//    }
//
//    fn parse(args: &[&str]) -> ValidationResult<(Arc<str>, Arc<str>, Arc<str>)> {
//        assert_eq!(3, args.len());
//        return Result::Ok((Arc::from(&*args[0]), Arc::from(&*args[1]), Arc::from(&*args[2])));
//    }
//}
