use std::sync::Arc;

// Thanks to [1] args go here...
//
// [1] https://github.com/rust-lang/rust/issues/54647

#[derive(RegistryArgs)]
pub struct ZeroRegistryArgs {
}

impl ZeroRegistryArgs {
    pub fn new() -> ZeroRegistryArgs {
        return ZeroRegistryArgs {
        };
    }
}

#[derive(RegistryArgs)]
pub struct OneKeyRegistryArgs {
    pub key: Arc<str>,
}

impl OneKeyRegistryArgs {
    pub fn new(s: &str) -> OneKeyRegistryArgs {
        return OneKeyRegistryArgs {
            key: Arc::from(s),
        }
    }
}
