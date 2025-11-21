use std::path::PathBuf;

use anyhow::{bail, Context};
use uuid::Uuid;

use crate::bases::Base;
use crate::orchestration::{
    MetricRecord, OrchestrationLog, WritingMetricKind, WritingMetricRecord,
};
use crate::storage::ai_layer::WritingAiStore;
use serde_json;

use super::WritingResult;

/// Payload persisted for undo operations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoPayload {
    pub target_file: PathBuf,
    pub previous_content: String,
}

/// Records an undo checkpoint for a section edit.
pub fn record_checkpoint(
    base: &Base,
    slug: &str,
    event_id: Uuid,
    payload: UndoPayload,
) -> WritingResult<PathBuf> {
    let store = WritingAiStore::new(base);
    let path = store.save_undo_payload(slug, &event_id.to_string(), &payload)?;
    prune_undo_history(base, slug)?;
    Ok(path)
}

/// Restores content from a prior checkpoint.
pub fn revert_checkpoint(base: &Base, slug: &str, event_id: &Uuid) -> WritingResult<UndoPayload> {
    let started = std::time::Instant::now();
    let store = WritingAiStore::new(base);
    let payload: Option<UndoPayload> = store.load_undo_payload(slug, &event_id.to_string())?;
    let payload = payload.with_context(|| format!("No undo checkpoint for event {}", event_id))?;
    if !payload.target_file.exists() {
        bail!(
            "Target file {} missing; cannot undo.",
            payload.target_file.display()
        );
    }
    std::fs::write(&payload.target_file, &payload.previous_content).with_context(|| {
        format!(
            "Failed to restore {} from checkpoint",
            payload.target_file.display()
        )
    })?;
    record_undo_metric(
        base,
        started.elapsed(),
        true,
        payload.target_file.display().to_string(),
    );
    Ok(payload)
}

fn prune_undo_history(base: &Base, slug: &str) -> WritingResult<()> {
    let store = WritingAiStore::new(base);
    let dir = store.project_root(slug).join("undo");
    if !dir.exists() {
        return Ok(());
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .collect();
    if entries.len() <= 20 {
        return Ok(());
    }
    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    while entries.len() > 20 {
        if let Some(entry) = entries.first() {
            let _ = std::fs::remove_file(entry.path());
        }
        entries.remove(0);
    }
    Ok(())
}

fn record_undo_metric(base: &Base, duration: std::time::Duration, success: bool, file: String) {
    let log = OrchestrationLog::for_base(base);
    let _ = log.record_metric(&MetricRecord::Writing(WritingMetricRecord {
        kind: WritingMetricKind::Undo,
        duration_ms: duration.as_millis() as i64,
        success,
        details: serde_json::json!({ "file": file }),
    }));
}
