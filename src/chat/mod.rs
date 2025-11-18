use crate::acquisition::{
    discover_candidates, generate_candidates_from_interview, run_acquisition_batch,
    undo_last_batch, CandidatePaper, InterviewAnswers,
};
use crate::bases::{Base, BaseManager};
use crate::ingestion::{format_batch_status, ingest_local_pdfs, IngestionRunner};
use crate::orchestration::{log_event, EventType, OrchestrationLog};
use crate::reports::generate_and_log_reports;
use anyhow::{Context, Result};
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

    /// Placeholder metadata refresh command wiring that keeps chat-first UX alive.
    pub fn metadata_refresh(&mut self, paper_ids: Option<Vec<Uuid>>) -> Result<String> {
        let scope = match paper_ids {
            Some(ref ids) if !ids.is_empty() => format!("{} papers", ids.len()),
            _ => "entire Base".to_string(),
        };
        Ok(format!(
            "Metadata refresh stub queued for {}. Enrichment pipeline will replace this once implemented.",
            scope
        ))
    }

    /// Placeholder consent-driven figure extraction command.
    pub fn figures_extract(&mut self, batch_hint: Option<Uuid>) -> Result<String> {
        let scope = batch_hint
            .map(|id| format!("batch {}", id))
            .unwrap_or_else(|| "latest ingestion batch".into());
        Ok(format!(
            "Figure extraction stub acknowledged for {}. Consent + storage workflow pending implementation.",
            scope
        ))
    }

    /// Returns a minimal orchestration history summary (ingestion/acquisition).
    pub fn history_show(&self, range_hint: Option<&str>) -> Result<Vec<String>> {
        let base = self.active_base()?;
        let log = OrchestrationLog::for_base(&base);
        let mut entries = Vec::new();
        for batch in log.load_batches()? {
            entries.push(format!(
                "{} | Acquisition batch {} (approved '{}')",
                batch.approved_at, batch.batch_id, batch.approved_text
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
}
