use std::sync::Arc;

pub type Line = Arc<str>;

pub trait Stream {
    fn write_line(&mut self, Line) -> bool;
    fn close(&mut self);
}
