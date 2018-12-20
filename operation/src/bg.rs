use Operation;
use StreamWrapper;
use std::collections::VecDeque;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["bg"];
}

#[derive(Default)]
pub struct Impl {
}

impl Operation for Impl {
    fn validate(&self, args: &mut VecDeque<String>) -> StreamWrapper {
        let name = args.pop_front().unwrap();
        let op = super::find(&name);
        let op = op.validate(args);

        return StreamWrapper::new(move |os| {
            return op.wrap(os);
        });
    }
}
