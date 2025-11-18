pub mod batch_store;
pub mod dedup;
pub mod error;
pub mod metadata;
pub mod runner;
pub mod status;

use crate::bases::{Base, BaseManager, LibraryEntry};
use crate::orchestration::{log_event, EventType};
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir;

pub use batch_store::{IngestionBatchState, IngestionBatchStatus, IngestionBatchStore};
pub use dedup::{
    detect_duplicate_groups, format_duplicate_group, merge_duplicate_group, DuplicateGroup,
};
pub use error::{IngestionIssue, IngestionIssueReason};
pub use metadata::{refresh_metadata, MetadataRefreshRequest, MetadataRefreshResult};
pub use runner::{IngestionOutcome, IngestionRunner};
pub use status::format_batch_status;

/// Result of a Path A ingestion run or an incremental runner chunk.
#[derive(Debug, Clone, Default)]
pub struct IngestionSummary {
    pub total_files: usize,
    pub ingested: usize,
    pub skipped: usize,
    pub failed: usize,
    pub issues: Vec<IngestionIssue>,
}

/// Ingests PDFs or export files from a directory into the Base.
pub fn ingest_local_pdfs<P: AsRef<Path>>(
    manager: &BaseManager,
    base: &Base,
    folder: P,
) -> Result<IngestionSummary> {
    let folder = folder.as_ref();
    let mut entries = manager.load_library_entries(base)?;
    let mut summary = IngestionSummary {
        total_files: 0,
        ingested: 0,
        skipped: 0,
        failed: 0,
        issues: Vec::new(),
    };

    if !folder.exists() {
        anyhow::bail!("Folder {:?} does not exist", folder);
    }

    for entry in walkdir::WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        summary.total_files += 1;
        let path = entry.path();
        if !is_supported_file(path) {
            summary.skipped += 1;
            continue;
        }
        let identifier = format!(
            "local:{}",
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        );
        if entries
            .iter()
            .any(|existing| existing.identifier == identifier)
        {
            summary.skipped += 1;
            continue;
        }

        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled file")
            .replace('_', " ");

        let mut library_entry = LibraryEntry {
            entry_id: Uuid::new_v4(),
            title,
            authors: Vec::new(),
            venue: Some("Local Import".into()),
            year: None,
            identifier: identifier.clone(),
            pdf_paths: vec![path.to_path_buf()],
            needs_pdf: false,
            notes: Some("Imported via Path A ingestion".into()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Copy to User Layer to keep everything under workspace
        let target_pdf = copy_to_user_layer(base, path)?;
        library_entry.pdf_paths = vec![target_pdf];

        entries.push(library_entry);
        summary.ingested += 1;
    }

    manager.save_library_entries(base, &entries)?;

    log_event(
        manager,
        base,
        EventType::PathAIngestion,
        json!({
            "folder": folder,
            "ingested": summary.ingested,
            "skipped": summary.skipped,
            "failed": summary.failed
        }),
    )?;

    Ok(summary)
}

pub(crate) fn is_supported_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(|s| s.to_lowercase()),
        Some(ref ext) if ["pdf", "txt", "md"].contains(&ext.as_str())
    )
}

pub(crate) fn copy_to_user_layer(base: &Base, path: &Path) -> Result<PathBuf> {
    let storage_dir = base.user_layer_path.join("imported");
    fs::create_dir_all(&storage_dir)?;
    let file_name = path.file_name().context("File missing name during copy")?;
    let target = storage_dir.join(file_name);
    fs::copy(path, &target)?;
    Ok(target)
}
