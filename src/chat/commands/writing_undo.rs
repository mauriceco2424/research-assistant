use anyhow::{Context, Result};
use serde_json::json;
use uuid::Uuid;

use crate::bases::Base;
use crate::orchestration::events::{log_writing_undo_applied, WritingEventDetails};
use crate::writing::undo::revert_checkpoint;

/// Reverts an edit using a stored undo checkpoint.
pub fn apply_undo(base: &Base, slug: &str, event_id: &str) -> Result<String> {
    let parsed =
        Uuid::parse_str(event_id).with_context(|| format!("Invalid event id '{}'", event_id))?;
    let payload = revert_checkpoint(base, slug, &parsed)?;
    let event = log_writing_undo_applied(
        base,
        WritingEventDetails::with_payload(
            slug,
            json!({
                "reverted_event": parsed,
                "file": payload.target_file.display().to_string(),
            }),
        )
        .with_files_touched([payload.target_file.display().to_string()]),
    )?;
    Ok(format!(
        "[OK] Reverted event {}. Restored file {} (event {}).",
        event_id,
        payload.target_file.display(),
        event
    ))
}
