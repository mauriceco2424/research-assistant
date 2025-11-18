use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum IngestionIssueReason {
    UnsupportedFormat,
    DuplicateIdentifier,
    ReadFailure,
    CopyFailure,
}

#[derive(Debug, Clone)]
pub struct IngestionIssue {
    pub path: PathBuf,
    pub reason: IngestionIssueReason,
    pub message: String,
}

impl IngestionIssue {
    pub fn new(path: PathBuf, reason: IngestionIssueReason, message: impl Into<String>) -> Self {
        Self {
            path,
            reason,
            message: message.into(),
        }
    }
}

pub(crate) fn is_supported_extension(path: &Path) -> bool {
    super::is_supported_file(path)
}
