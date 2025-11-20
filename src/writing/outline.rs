use super::{not_implemented, WritingResult};

#[derive(Debug, Default)]
pub struct Outline;

pub fn generate_outline() -> WritingResult<Outline> {
    Err(not_implemented("generate_outline"))
}

pub fn persist_outline(_outline: &Outline) -> WritingResult<()> {
    Err(not_implemented("persist_outline"))
}
