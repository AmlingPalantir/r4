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
