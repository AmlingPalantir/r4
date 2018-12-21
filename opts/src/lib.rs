use std::collections::HashMap;

fn name_from_arg(name: &str) -> Option<&str> {
    if name.starts_with("--") {
        return Some(&name[2..]);
    }
    if name.starts_with("-") {
        return Some(&name[1..]);
    }
    return None;
}

pub fn parse<P>(args: &mut Vec<String>, p: &mut P, opts: Vec<(&str, usize, Box<Fn(&mut P, &[String])>)>) {
    let m: HashMap<&str, _> = opts.iter().map(|(alias, argct, f)| (*alias, (*argct, f))).collect();

    let mut save_index = 0;
    let mut next_index = 0;
    loop {
        if next_index == args.len() {
            args.truncate(save_index);
            return;
        }

        if let Some(name) = name_from_arg(&args[next_index]) {
            match m.get(name) {
                Some((argct, f)) => {
                    let start = next_index + 1;
                    let end = start + argct;
                    if end > args.len() {
                        panic!();
                    }
                    f(p, &args[start..end]);
                    next_index = end;
                    continue;
                }
                None => {
                    panic!();
                }
            }
        }

        args.swap(save_index, next_index);
        save_index += 1;
        next_index += 1;
    }
}

pub trait OptionTrait {
    type PreType;
    type ValType;

    fn argct() -> usize;
    fn set(&mut Self::PreType, &[String]);
    fn val(Self::PreType) -> Self::ValType;
}

pub enum StringOption {
}

impl OptionTrait for StringOption {
    type PreType = Option<String>;
    type ValType = String;

    fn argct() -> usize {
        return 1;
    }

    fn set(p: &mut Option<String>, a: &[String]) {
        if let Some(_) = *p {
            panic!();
        }
        *p = Some(a[0].clone());
    }

    fn val(p: Option<String>) -> String {
        return p.unwrap();
    }
}

#[macro_export]
macro_rules! parse_opt {
    {$args:ident, $(($alias:expr, $type:ty, $f:ident)),*,} => {
        #[derive(Default)]
        struct Pre {
            $(
                $f: <$type as $crate::OptionTrait>::PreType,
            )*
        }
        let mut p = Pre::default();
        opts::parse($args, &mut p, vec![
            $(
                (
                    $alias,
                    <$type as $crate::OptionTrait>::argct(),
                    Box::new(|p: &mut Pre, a: &[String]| {
                        <$type as $crate::OptionTrait>::set(&mut p.$f, a)
                    }),
                ),
            ),*
        ]);
        $(
            let $f = <$type as $crate::OptionTrait>::val(p.$f);
        );*
    }
}
