use super::{not_implemented, WritingResult};

pub fn record_checkpoint() -> WritingResult<()> {
    Err(not_implemented("record_checkpoint"))
}

pub fn revert_checkpoint() -> WritingResult<()> {
    Err(not_implemented("revert_checkpoint"))
}
