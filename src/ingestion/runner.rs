use super::batch_store::{IngestionBatchState, IngestionBatchStatus, IngestionBatchStore};
use super::error::{is_supported_extension, IngestionIssue, IngestionIssueReason};
use super::IngestionSummary;
use crate::bases::{Base, BaseManager, LibraryEntry};
use crate::ingestion::copy_to_user_layer;
use crate::orchestration::{
    log_event, EventType, IngestionMetricsRecord, MetricRecord, OrchestrationLog,
    INGESTION_SLA_SECS,
};
use anyhow::{Context, Result};
use chrono::Utc;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use serde_json::json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

/// Outcome of an ingestion runner invocation (chunk or completion).
pub struct IngestionOutcome {
    pub state: IngestionBatchState,
    pub summary: IngestionSummary,
}

impl IngestionOutcome {
    pub fn describe_for_chat(&self) -> String {
        let status = format!("{:?}", self.state.status);
        let mut line = format!(
            "Batch {} ({status}) processed {} files this round (ingested {}, skipped {}, failed {}).",
            self.state.batch_id,
            self.summary.total_files,
            self.summary.ingested,
            self.summary.skipped,
            self.summary.failed
        );
        if !self.summary.issues.is_empty() {
            let first = &self.summary.issues[0];
            line.push_str(&format!(
                " Example issue: {:?} at {}.",
                first.reason,
                first.path.display()
            ));
        }
        if let Some(cp) = &self.state.last_checkpoint {
            line.push_str(&format!(" Checkpoint saved at {cp}."));
        }
        line
    }
}

pub struct IngestionRunner<'a> {
    manager: &'a BaseManager,
    base: Base,
    store: IngestionBatchStore,
    checkpoint_interval: usize,
}

impl<'a> IngestionRunner<'a> {
    pub fn new(manager: &'a BaseManager, base: Base) -> Self {
        let checkpoint_interval =
            manager.config.ingestion.checkpoint_interval_files.max(1) as usize;
        let store = IngestionBatchStore::for_base(&base);
        Self {
            manager,
            base,
            store,
            checkpoint_interval,
        }
    }

    pub fn start_batch<P: AsRef<Path>>(&self, folder: P) -> Result<IngestionOutcome> {
        let folder = folder.as_ref();
        if !folder.exists() {
            anyhow::bail!("Folder {:?} does not exist", folder);
        }
        let files = enumerate_supported_files(folder)?;
        let mut state = IngestionBatchState::new(
            &self.base,
            folder.to_path_buf(),
            self.manager.config.ingestion.remote_metadata_allowed,
        );
        state.total_files = files.len() as u64;
        self.store.append(&state)?;
        log_event(
            self.manager,
            &self.base,
            EventType::IngestionBatchStarted,
            json!({
                "batch_id": state.batch_id,
                "source": folder,
                "total_files": state.total_files
            }),
        )?;
        self.process_and_persist(state, files)
    }

    pub fn resume_latest(&self) -> Result<IngestionOutcome> {
        let state = self
            .store
            .latest()?
            .context("No ingestion batches found for this base")?;
        self.resume_from_state(state)
    }

    pub fn resume_batch(&self, batch_id: Uuid) -> Result<IngestionOutcome> {
        let state = self
            .store
            .get(&batch_id)?
            .context("Batch not found, cannot resume")?;
        self.resume_from_state(state)
    }

    pub fn pause_latest(&self) -> Result<bool> {
        if let Some(mut state) = self.store.latest()? {
            if matches!(state.status, IngestionBatchStatus::Running) {
                state.mark_paused(state.last_checkpoint.clone());
                self.store.upsert(&state)?;
                log_event(
                    self.manager,
                    &self.base,
                    EventType::IngestionBatchPaused,
                    json!({ "batch_id": state.batch_id }),
                )?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn latest_state(&self) -> Result<Option<IngestionBatchState>> {
        self.store.latest()
    }

    fn resume_from_state(&self, mut state: IngestionBatchState) -> Result<IngestionOutcome> {
        match state.status {
            IngestionBatchStatus::Completed => {
                anyhow::bail!("Batch {} is already complete.", state.batch_id);
            }
            IngestionBatchStatus::Failed => {
                anyhow::bail!(
                    "Batch {} failed earlier and needs manual intervention.",
                    state.batch_id
                );
            }
            _ => {}
        }
        let folder = state.source_path.clone();
        if !folder.exists() {
            anyhow::bail!(
                "Ingestion folder {:?} no longer exists; cannot resume.",
                folder
            );
        }
        let files = enumerate_supported_files(&folder)?;
        state.total_files = files.len() as u64;
        self.process_and_persist(state, files)
    }

    fn process_and_persist(
        &self,
        mut state: IngestionBatchState,
        files: Vec<PathBuf>,
    ) -> Result<IngestionOutcome> {
        let summary = self.process_chunk(&mut state, &files)?;
        self.store.upsert(&state)?;
        if matches!(state.status, IngestionBatchStatus::Completed) {
            let duration_ms = (Utc::now() - state.started_at).num_milliseconds();
            let sla_breached = duration_ms > INGESTION_SLA_SECS * 1000;
            log_event(
                self.manager,
                &self.base,
                EventType::IngestionBatchCompleted,
                json!({
                    "batch_id": state.batch_id,
                    "ingested": state.ingested_files,
                    "skipped": state.skipped_files,
                    "failed": state.failed_files,
                    "duration_ms": duration_ms,
                    "sla_breached": sla_breached
                }),
            )?;
            let log = OrchestrationLog::for_base(&self.base);
            log.record_metric(&MetricRecord::Ingestion(IngestionMetricsRecord {
                batch_id: state.batch_id,
                duration_ms,
                ingested: state.ingested_files,
                skipped: state.skipped_files,
                failed: state.failed_files,
                sla_breached,
            }))?;
        }
        Ok(IngestionOutcome { state, summary })
    }

fn process_chunk(
        &self,
        state: &mut IngestionBatchState,
        files: &[PathBuf],
    ) -> Result<IngestionSummary> {
        let mut summary = IngestionSummary::default();
        state.mark_running();
        let mut entries = self.manager.load_library_entries(&self.base)?;
        let mut known_identifiers: HashSet<String> =
            entries.iter().map(|e| e.identifier.clone()).collect();
        let start_index = starting_index(files, &state.last_checkpoint, &state.source_path);
        let mut processed_in_chunk = 0usize;
        let mut last_processed_label: Option<String> = None;
        let mut pending_entries: Vec<PendingEntry> = Vec::new();

        for path in files.iter().skip(start_index) {
            if processed_in_chunk >= self.checkpoint_interval {
                break;
            }
            summary.total_files += 1;
            state.processed_files += 1;
            processed_in_chunk += 1;

            if !is_supported_extension(path) {
                summary.skipped += 1;
                summary.failed += 1;
                summary.issues.push(IngestionIssue::new(
                    path.clone(),
                    IngestionIssueReason::UnsupportedFormat,
                    "Unsupported file extension",
                ));
                continue;
            }

            let identifier = format!(
                "local:{}",
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
            );
            if known_identifiers.contains(&identifier) {
                summary.skipped += 1;
                summary.issues.push(IngestionIssue::new(
                    path.clone(),
                    IngestionIssueReason::DuplicateIdentifier,
                    "Identifier already exists in this base",
                ));
                continue;
            }

            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled file")
                .replace('_', " ");
            let entry = LibraryEntry {
                entry_id: Uuid::new_v4(),
                title,
                authors: Vec::new(),
                venue: Some("Local Import".into()),
                year: None,
                identifier: identifier.clone(),
                pdf_paths: Vec::new(),
                needs_pdf: false,
                notes: Some("Imported via Ingestion Runner".into()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            pending_entries.push(PendingEntry {
                path: path.clone(),
                identifier,
                entry,
            });
        }

        if !pending_entries.is_empty() {
            let concurrency = self
                .manager
                .config
                .ingestion
                .max_parallel_file_copies
                .max(1) as usize;
            let pool = ThreadPoolBuilder::new()
                .num_threads(concurrency)
                .build()
                .context("Failed to configure ingestion copy thread pool")?;
            let completed = pool.install(|| {
                pending_entries
                    .into_par_iter()
                    .map(|pending| {
                        let result = copy_to_user_layer(&self.base, &pending.path);
                        (pending, result)
                    })
                    .collect::<Vec<_>>()
            });

            for (mut pending, result) in completed {
                if known_identifiers.contains(&pending.identifier) {
                    summary.skipped += 1;
                    summary.issues.push(IngestionIssue::new(
                        pending.path.clone(),
                        IngestionIssueReason::DuplicateIdentifier,
                        "Identifier already exists in this base",
                    ));
                    continue;
                }
                match result {
                    Ok(target_pdf) => {
                        pending.entry.pdf_paths = vec![target_pdf];
                        entries.push(pending.entry);
                        known_identifiers.insert(pending.identifier);
                        summary.ingested += 1;
                        last_processed_label =
                            Some(relative_label(&pending.path, &state.source_path).unwrap_or_default());
                    }
                    Err(err) => {
                        summary.failed += 1;
                        summary.issues.push(IngestionIssue::new(
                            pending.path.clone(),
                            IngestionIssueReason::CopyFailure,
                            format!("Unable to copy file: {err}"),
                        ));
                    }
                }
            }
        }

        self.manager.save_library_entries(&self.base, &entries)?;

        state.ingested_files += summary.ingested as u64;
        state.skipped_files += summary.skipped as u64;
        state.failed_files += summary.failed as u64;

        if let Some(label) = last_processed_label {
            state.last_checkpoint = Some(label);
        }

        if (state.processed_files as usize) >= files.len() {
            state.mark_completed();
            state.last_checkpoint = None;
        } else if state.processed_files > 0 && processed_in_chunk >= self.checkpoint_interval {
            state.mark_paused(state.last_checkpoint.clone());
        } else if matches!(state.status, IngestionBatchStatus::Pending) {
            state.mark_running();
        }

        Ok(summary)
    }
}

struct PendingEntry {
    path: PathBuf,
    identifier: String,
    entry: LibraryEntry,
}

fn enumerate_supported_files(folder: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && is_supported_extension(entry.path()) {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}

fn starting_index(files: &[PathBuf], checkpoint: &Option<String>, root: &Path) -> usize {
    if let Some(label) = checkpoint {
        if let Some(pos) = files
            .iter()
            .position(|p| relative_label(p, root).as_ref() == Some(label))
        {
            return pos + 1;
        }
    }
    0
}

fn relative_label(path: &Path, root: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}
