use crate::bases::Base;
use std::path::PathBuf;

/// Computes a stable metadata-only storage path for a discovery record.
pub fn metadata_entry_path(base: &Base, identifier: &str) -> PathBuf {
    let safe = identifier.replace(['/', ':'], "_");
    base.user_layer_path
        .join("metadata")
        .join(format!("{safe}.json"))
}

/// Computes a placeholder PDF path for discovery acquisitions.
pub fn pdf_placeholder_path(base: &Base, identifier: &str) -> PathBuf {
    let safe = identifier.replace(['/', ':'], "_");
    base.user_layer_path
        .join("pdfs")
        .join(format!("{safe}.pdf"))
}
