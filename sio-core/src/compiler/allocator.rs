use crate::compiler::value::{ThreadValue};
use werbolg_exec::WAllocator;

pub struct ThreadAllocator;
impl WAllocator for ThreadAllocator {
    type Value = ThreadValue;
}
