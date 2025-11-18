use crate::acquisition::{
    discover_candidates, generate_candidates_from_interview, run_acquisition_batch,
    run_figure_extraction, undo_last_batch, CandidatePaper, InterviewAnswers,
};
use crate::bases::{Base, BaseManager};
use crate::ingestion::{
    detect_duplicate_groups, format_batch_status, format_duplicate_group, ingest_local_pdfs,
    merge_duplicate_group, refresh_metadata, IngestionRunner, MetadataRefreshRequest,
};
use crate::orchestration::{
    log_event, require_remote_operation_consent, ConsentOperation, ConsentScope, EventType,
    OrchestrationLog,
};
use crate::reports::generate_and_log_reports;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Lightweight facade that emulates chat commands.
pub struct ChatSession {
    manager: BaseManager,
    pending_candidates: HashMap<Uuid, Vec<CandidatePaper>>,
}

impl ChatSession {
    pub fn new() -> Result<Self> {
        Ok(Self {
            manager: BaseManager::new()?,
            pending_candidates: HashMap::new(),
        })
    }

    fn active_base(&self) -> Result<Base> {
        self.manager
            .active_base()?
            .context("No active Base. Create or select a Base first.")
    }

    pub fn create_base(&mut self, name: &str) -> Result<Base> {
        let base = self.manager.create_base(name)?;
        Ok(base)
    }

    pub fn list_bases(&self) -> Result<Vec<Base>> {
        self.manager.list_bases()
    }

    pub fn select_base(&mut self, base_id: &Uuid) -> Result<()> {
        self.manager.set_active_base(base_id)
    }

    pub fn ingest_path_a<P: AsRef<Path>>(&mut self, folder: P) -> Result<String> {
        let base = self.active_base()?;
        let summary = ingest_local_pdfs(&self.manager, &base, folder)?;
        let entries = self.manager.load_library_entries(&base)?;
        let _ = generate_and_log_reports(&self.manager, &base, &entries)?;
        Ok(format!(
            "Ingested {} files (skipped {}). Reports updated.",
            summary.ingested, summary.skipped
        ))
    }

    pub fn ingest_start<P: AsRef<Path>>(&mut self, folder: P) -> Result<String> {
        let base = self.active_base()?;
        let runner = IngestionRunner::new(&self.manager, base.clone());
        let outcome = runner.start_batch(folder.as_ref())?;
        Ok(outcome.describe_for_chat())
    }

    pub fn ingest_status(&mut self) -> Result<String> {
        let base = self.active_base()?;
        let runner = IngestionRunner::new(&self.manager, base);
        if let Some(state) = runner.latest_state()? {
            Ok(format_batch_status(&state))
        } else {
            Ok("No ingestion batches recorded for this Base.".into())
        }
    }

    pub fn ingest_pause(&mut self) -> Result<String> {
        let base = self.active_base()?;
        let runner = IngestionRunner::new(&self.manager, base);
        if runner.pause_latest()? {
            Ok("Pause requested for the active ingestion batch.".into())
        } else {
            anyhow::bail!("No running ingestion batch to pause.");
        }
    }

    pub fn ingest_resume(&mut self) -> Result<String> {
        let base = self.active_base()?;
        let runner = IngestionRunner::new(&self.manager, base);
        let outcome = runner.resume_latest()?;
        Ok(outcome.describe_for_chat())
    }

    pub fn metadata_refresh(
        &mut self,
        paper_ids: Option<Vec<Uuid>>,
        approval_text: Option<&str>,
    ) -> Result<String> {
        let base = self.active_base()?;
        let remote_allowed = self.manager.config.ingestion.remote_metadata_allowed;
        let allow_remote = remote_allowed && approval_text.is_some();
        if remote_allowed && approval_text.is_none() {
            return Err(anyhow::anyhow!(
                "Remote metadata lookup requires explicit approval text."
            ));
        }
        if allow_remote {
            let scope = ConsentScope {
                batch_id: None,
                paper_ids: paper_ids.clone().unwrap_or_default(),
            };
            require_remote_operation_consent(
                &self.manager,
                &base,
                ConsentOperation::MetadataLookup,
                approval_text.unwrap(),
                scope,
                serde_json::json!({ "count": paper_ids.as_ref().map(|v| v.len()).unwrap_or(0) }),
            )?;
        }
        let request = MetadataRefreshRequest {
            paper_ids,
            allow_remote,
            approval_text: approval_text.map(|s| s.to_string()),
        };
        let outcome = refresh_metadata(&self.manager, &base, request)?;
        let mut response = format!(
            "Metadata refresh batch {} updated {} records.",
            outcome.batch_id,
            outcome.updated_records.len()
        );
        if outcome.offline_mode {
            response.push_str(" Remote lookups disabled; used offline heuristics.");
        } else {
            response.push_str(" Remote lookups approved and applied.");
        }
        if !outcome.duplicates.is_empty() {
            response.push_str(&format!(
                " Found {} duplicate DOI groups. Use metadata_list_duplicates for details.",
                outcome.duplicates.len()
            ));
        }
        response.push_str(&format!(
            " DOI coverage: {:.1}% ({} of {}).",
            outcome.doi_accuracy,
            outcome.doi_assigned,
            outcome.updated_records.len()
        ));
        if outcome.manual_review_backlog > 0 {
            response.push_str(&format!(
                " {} entries need manual DOI review.",
                outcome.manual_review_backlog
            ));
        }
        Ok(response)
    }

    pub fn metadata_list_duplicates(&mut self) -> Result<Vec<String>> {
        let base = self.active_base()?;
        let records = self.manager.load_metadata_records(&base)?;
        Ok(detect_duplicate_groups(&records)
            .into_iter()
            .map(|group| format_duplicate_group(&group))
            .collect())
    }

    pub fn metadata_merge_duplicate(&mut self, doi: &str, keep_record_id: Uuid) -> Result<String> {
        let base = self.active_base()?;
        let removed = merge_duplicate_group(&self.manager, &base, doi, keep_record_id)?;
        Ok(format!(
            "Merged duplicate DOI {} by keeping {} (removed {}).",
            doi, keep_record_id, removed
        ))
    }

    pub fn undo_last_metadata_refresh(&mut self) -> Result<String> {
        let base = self.active_base()?;
        if let Some(batch) = self.manager.undo_last_metadata_change_batch(&base)? {
            log_event(
                &self.manager,
                &base,
                EventType::MetadataRefreshUndo,
                serde_json::json!({ "undo_batch": batch.batch_id }),
            )?;
            Ok(format!(
                "Reverted metadata batch {} affecting {} records.",
                batch.batch_id,
                batch.changes.len()
            ))
        } else {
            Ok("No metadata refresh batches to undo.".into())
        }
    }

    pub fn figures_extract(
        &mut self,
        paper_ids: Option<Vec<Uuid>>,
        approval_text: &str,
    ) -> Result<String> {
        if approval_text.trim().is_empty() {
            anyhow::bail!("Figure extraction requires approval text.");
        }
        let base = self.active_base()?;
        let scope = ConsentScope {
            batch_id: None,
            paper_ids: paper_ids.clone().unwrap_or_default(),
        };
        require_remote_operation_consent(
            &self.manager,
            &base,
            ConsentOperation::FigureExtraction,
            approval_text,
            scope,
            serde_json::json!({ "papers": paper_ids.as_ref().map(|v| v.len()).unwrap_or(0) }),
        )?;
        let outcome = run_figure_extraction(&self.manager, &base, paper_ids, approval_text)?;
        let entries = self.manager.load_library_entries(&base)?;
        let _ = generate_and_log_reports(&self.manager, &base, &entries)?;
        Ok(format!(
            "Figure extraction batch {} created {} figure assets.",
            outcome.batch.batch_id,
            outcome.records.len()
        ))
    }

    /// Returns a minimal orchestration history summary (ingestion/acquisition).
    pub fn history_show(&self, range_hint: Option<&str>) -> Result<Vec<String>> {
        let base = self.active_base()?;
        let log = OrchestrationLog::for_base(&base);
        let cutoff = parse_history_range(range_hint)?;
        let mut entries = Vec::new();
        for batch in log.load_batches()? {
            if batch.approved_at >= cutoff {
                entries.push(format!(
                    "{} | Acquisition batch {} (approved '{}')",
                    batch.approved_at, batch.batch_id, batch.approved_text
                ));
            }
        }
        for batch in log.load_figure_batches()? {
            if batch.approved_at >= cutoff {
                entries.push(format!(
                    "{} | Figure extraction {} ({} assets)",
                    batch.approved_at,
                    batch.batch_id,
                    batch.figure_asset_ids.len()
                ));
            }
        }
        for event in log.load_events_since(cutoff)? {
            entries.push(format!(
                "{} | Event {:?} -> {}",
                event.timestamp, event.event_type, event.details
            ));
        }
        if entries.is_empty() {
            entries.push(format!(
                "No orchestration history yet{}.",
                range_hint
                    .map(|hint| format!(" for range '{}'", hint))
                    .unwrap_or_default()
            ));
        }
        Ok(entries)
    }

    pub fn path_b_interview(&mut self, answers: InterviewAnswers) -> Result<Vec<CandidatePaper>> {
        let base = self.active_base()?;
        let candidates = generate_candidates_from_interview(&answers);
        self.pending_candidates.insert(base.id, candidates.clone());
        log_event(
            &self.manager,
            &base,
            EventType::PathBInterview,
            serde_json::json!({
                "topic": answers.topic,
                "question_count": answers.research_questions.len()
            }),
        )?;
        Ok(candidates)
    }

    pub fn approve_candidates(
        &mut self,
        selected_identifiers: &[String],
        approval_text: &str,
    ) -> Result<String> {
        let base = self.active_base()?;
        let candidates = self
            .pending_candidates
            .get(&base.id)
            .cloned()
            .unwrap_or_default();
        let selected: Vec<CandidatePaper> = candidates
            .into_iter()
            .filter(|c| selected_identifiers.contains(&c.identifier))
            .collect();
        if selected.is_empty() {
            anyhow::bail!("No matching candidates selected");
        }
        let batch = run_acquisition_batch(&self.manager, &base, &selected, approval_text)?;
        let entries = self.manager.load_library_entries(&base)?;
        let (with_pdf, needs_pdf): (Vec<_>, Vec<_>) = selected.iter().partition(|c| c.open_access);
        generate_and_log_reports(&self.manager, &base, &entries)?;
        Ok(format!(
            "Batch {} added {} papers ({} need manual PDFs).\nWith PDFs: {}\nNeeds PDFs: {}",
            batch.batch_id,
            selected.len(),
            needs_pdf.len(),
            with_pdf
                .iter()
                .map(|c| c.metadata_summary())
                .collect::<Vec<_>>()
                .join(", "),
            needs_pdf
                .iter()
                .map(|c| c.metadata_summary())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    pub fn discover_and_add(
        &mut self,
        topic: &str,
        count: usize,
        approval_text: &str,
    ) -> Result<String> {
        let base = self.active_base()?;
        let candidates = discover_candidates(topic, count);
        let batch = run_acquisition_batch(&self.manager, &base, &candidates, approval_text)?;
        let entries = self.manager.load_library_entries(&base)?;
        generate_and_log_reports(&self.manager, &base, &entries)?;
        Ok(format!(
            "Discovery batch {} added {} candidates for topic '{}'.",
            batch.batch_id,
            candidates.len(),
            topic
        ))
    }

    pub fn acquisition_history(&self) -> Result<Vec<String>> {
        let base = self.active_base()?;
        let log = OrchestrationLog::for_base(&base);
        let mut summaries = Vec::new();
        for batch in log.load_batches()? {
            summaries.push(format!(
                "{} - {} selections (approved '{}')",
                batch.approved_at,
                batch.records.len(),
                batch.approved_text
            ));
        }
        Ok(summaries)
    }

    pub fn undo_last_acquisition(&mut self) -> Result<Option<Uuid>> {
        let base = self.active_base()?;
        if let Some(batch) = undo_last_batch(&self.manager, &base)? {
            Ok(Some(batch.batch_id))
        } else {
            Ok(None)
        }
    }

    pub fn undo_last_figure_extraction(&mut self) -> Result<String> {
        let base = self.active_base()?;
        let log = OrchestrationLog::for_base(&base);
        if let Some(batch) = log.undo_last_figure_batch()? {
            let store = crate::acquisition::FigureStore::new(&base);
            let removed = store.remove_records_for_batch(&batch.batch_id)?;
            Ok(format!(
                "Undid figure extraction batch {} (removed {} assets).",
                batch.batch_id,
                removed.len()
            ))
        } else {
            Ok("No figure extraction batches to undo.".into())
        }
    }

    pub fn reprocess_figures(&mut self, paper_id: Uuid, approval_text: &str) -> Result<String> {
        if approval_text.trim().is_empty() {
            anyhow::bail!("Figure extraction requires approval text.");
        }
        let base = self.active_base()?;
        let scope = ConsentScope {
            batch_id: None,
            paper_ids: vec![paper_id],
        };
        require_remote_operation_consent(
            &self.manager,
            &base,
            ConsentOperation::FigureExtraction,
            approval_text,
            scope,
            serde_json::json!({ "paper": paper_id }),
        )?;
        let outcome =
            run_figure_extraction(&self.manager, &base, Some(vec![paper_id]), approval_text)?;
        Ok(format!(
            "Reprocessed figures for paper {} in batch {}.",
            paper_id, outcome.batch.batch_id
        ))
    }

    pub fn reprocess_metadata(
        &mut self,
        paper_id: Uuid,
        approval_text: Option<&str>,
    ) -> Result<String> {
        let base = self.active_base()?;
        let request = MetadataRefreshRequest {
            paper_ids: Some(vec![paper_id]),
            allow_remote: approval_text.is_some(),
            approval_text: approval_text.map(|s| s.to_string()),
        };
        let outcome = refresh_metadata(&self.manager, &base, request)?;
        Ok(format!(
            "Reprocessed metadata for {} (batch {}, remote={}).",
            paper_id, outcome.batch_id, outcome.used_remote
        ))
    }
}

fn parse_history_range(range_hint: Option<&str>) -> Result<DateTime<Utc>> {
    if let Some(hint) = range_hint {
        if let Some(days) = hint.strip_suffix('d') {
            if let Ok(num) = days.parse::<i64>() {
                return Ok(Utc::now() - chrono::Duration::days(num));
            }
        } else if let Some(hours) = hint.strip_suffix('h') {
            if let Ok(num) = hours.parse::<i64>() {
                return Ok(Utc::now() - chrono::Duration::hours(num));
            }
        }
    }
    Ok(Utc::now() - chrono::Duration::days(7))
}
