use Operation;
use StreamWrapper;
use record::FromPrimitive;
use record::Record;
use std::collections::VecDeque;
use std::sync::Arc;
use stream::Entry;

pub(crate) fn names() -> Vec<&'static str> {
    return vec!["test"];
}

#[derive(Default)]
pub struct Impl {
}

impl Operation for Impl {
    fn validate(&self, args: &mut VecDeque<String>) -> StreamWrapper {
        let msg: Arc<str> = Arc::from(args.pop_front().unwrap());

        return StreamWrapper::new(move |os| {
            let mut n = 0;
            let msg = msg.clone();

            return stream::transform(os, move |e| {
                let mut r = e.to_record();

                n += 1;
                r.set_path("n", Record::from_primitive(n));
                r.set_path("msg", Record::from_primitive_string(msg.clone()));

                return Entry::Record(r);
            });
        });
    }
}
