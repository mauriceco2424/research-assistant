use anyhow::{bail, Context, Result};
use serde_json::json;
use uuid::Uuid;

use crate::bases::{Base, BaseManager};
use crate::orchestration::events::{log_citation_flagged, log_section_edited, WritingEventDetails};
use crate::writing::citations::{verify_citations, CitationStatus};
use crate::writing::drafting::{apply_edit, DraftOutcome};
use crate::writing::undo::{record_checkpoint, UndoPayload};

/// Request to apply an inline edit to a draft section.
pub struct InlineEditRequest {
    pub project_slug: String,
    pub section_id: String,
    pub command: String,
    pub replacement: Option<String>,
}

/// Applies an inline edit and logs orchestration + citation events.
pub fn handle_inline_edit(
    manager: &BaseManager,
    base: &Base,
    request: InlineEditRequest,
) -> Result<String> {
    if request.command.trim().is_empty() {
        bail!("Edit command cannot be empty.");
    }
    if request.section_id.trim().is_empty() {
        bail!("Section id is required for inline edits.");
    }

    // Use event id to track undo and logging.
    let event_id = Uuid::new_v4();
    let section_uuid = Uuid::parse_str(&request.section_id)
        .with_context(|| format!("Invalid section id '{}'", request.section_id))?;
    let paths = crate::writing::project::ProjectPaths::new(base, &request.project_slug);
    let file_path = paths
        .sections_dir
        .join(format!("{}.tex", request.section_id));
    if !file_path.exists() {
        bail!(
            "Section {} not found under {}; generate drafts first.",
            request.section_id,
            paths.sections_dir.display()
        );
    }

    let before = std::fs::read_to_string(&file_path).unwrap_or_default();
    let checkpoint_path = record_checkpoint(
        base,
        &request.project_slug,
        event_id,
        UndoPayload {
            target_file: file_path.clone(),
            previous_content: before.clone(),
        },
    )?;

    // Simplified editing: append the command as a note or replace content when provided.
    let new_content = if let Some(body) = &request.replacement {
        body.clone()
    } else {
        let mut combined = before.clone();
        if !combined.ends_with('\n') {
            combined.push('\n');
        }
        combined.push_str("% Inline edit request:\n");
        combined.push_str(&request.command);
        combined.push('\n');
        combined
    };

    let outcome = apply_edit(
        base,
        &request.project_slug,
        section_uuid,
        &new_content,
        Some(event_id),
    )?;
    log_edit_events(
        manager,
        base,
        &request.project_slug,
        &outcome,
        Some(checkpoint_path),
    )?;

    Ok(format!(
        "[OK] Edit applied to section {}. Undo token: {}. Diff: {} bytes changed.",
        request.section_id,
        event_id,
        diff_len(&before, &outcome.content)
    ))
}

fn log_edit_events(
    manager: &BaseManager,
    base: &Base,
    slug: &str,
    outcome: &DraftOutcome,
    checkpoint: Option<std::path::PathBuf>,
) -> Result<()> {
    let details = WritingEventDetails::with_payload(
        slug,
        json!({
            "section_id": outcome.metadata.section_id,
            "citations": outcome.metadata.citations,
        }),
    )
    .with_files_touched([
        outcome.metadata.file_path.clone(),
        outcome.bibliography_path.display().to_string(),
    ]);
    let details = if let Some(path) = checkpoint {
        details.with_undo_checkpoint(path.display().to_string())
    } else {
        details
    };
    log_section_edited(base, details)?;

    // Flag unverified citations.
    let links = verify_citations(manager, base, &outcome.metadata.citations)?;
    let flagged: Vec<_> = links
        .into_iter()
        .filter(|link| link.status != CitationStatus::Verified)
        .collect();
    if !flagged.is_empty() {
        log_citation_flagged(
            base,
            WritingEventDetails::with_payload(slug, json!({ "flagged": flagged }))
                .with_files_touched([outcome.bibliography_path.display().to_string()]),
        )?;
    }
    Ok(())
}

fn diff_len(before: &str, after: &str) -> usize {
    before.len().abs_diff(after.len())
}
