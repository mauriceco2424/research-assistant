use crate::acquisition::{
    discover_candidates, generate_candidates_from_interview, run_acquisition_batch,
    run_figure_extraction, undo_last_batch, CandidatePaper, InterviewAnswers,
};
use crate::bases::{
    apply_narrative_update, category_slug, merge_categories, move_papers, AssignmentSource,
    AssignmentStatus, Base, BaseManager, CategoryAssignment, CategoryAssignmentsIndex,
    CategoryMetricsStore, CategoryOrigin, CategoryProposalStore, CategoryRecord,
    CategorySnapshotStore, CategoryStore, MergeOptions, NarrativeUpdate,
};
use crate::ingestion::{
    detect_duplicate_groups, format_batch_status, format_duplicate_group, ingest_local_pdfs,
    merge_duplicate_group, refresh_metadata, IngestionRunner, MetadataRefreshRequest,
};
use crate::orchestration::{
    log_event, require_remote_operation_consent, CategoryEditEventDetails, CategoryEditType,
    CategoryProposalEvent, ConsentOperation, ConsentScope, EventType, OrchestrationLog,
};
use crate::reports::{
    categorization::{proposals::CategoryProposalWorker, split, status::CategoryMetricsCollector},
    generate_and_log_reports,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
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

    pub fn categories_propose(&mut self, remote_summary_approval: Option<&str>) -> Result<String> {
        let base = self.active_base()?;
        let entries = self.manager.load_library_entries(&base)?;
        if entries.len() < 2 {
            anyhow::bail!("Need at least two papers in the Base before proposing categories.");
        }
        let mut consent_manifest_id = None;
        if let Some(approval) = remote_summary_approval {
            if approval.trim().is_empty() {
                anyhow::bail!("Remote narrative assistance requires approval text.");
            }
            let scope = ConsentScope {
                batch_id: None,
                paper_ids: Vec::new(),
            };
            let manifest = require_remote_operation_consent(
                &self.manager,
                &base,
                ConsentOperation::CategoryNarrativeSuggest,
                approval,
                scope,
                serde_json::json!({ "command": "categories_propose" }),
            )?;
            consent_manifest_id = Some(manifest.manifest_id);
        }

        let categorization_cfg = self.manager.config.categorization.clone();
        let worker = CategoryProposalWorker::new(
            categorization_cfg.max_proposals as usize,
            categorization_cfg.timeout_ms,
        );
        let started = Instant::now();
        let proposals = worker.generate(&base, &entries)?;
        if proposals.is_empty() {
            return Ok("No cohesive clusters discovered yet. Add more papers or adjust metadata before retrying.".into());
        }
        let duration_ms = started.elapsed().as_millis() as i64;
        let store = CategoryProposalStore::new(&base)?;
        let batch = store.save_batch(proposals, Some(duration_ms))?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_proposals_generated(
            &base,
            CategoryProposalEvent {
                batch_id: batch.batch_id,
                proposed_count: batch.proposals.len(),
                accepted_count: 0,
                rejected_count: 0,
                duration_ms: batch.duration_ms,
                consent_manifest_id,
            },
        )?;
        log.record_category_operation_metrics(
            "categories_propose",
            duration_ms,
            true,
            serde_json::json!({ "proposal_count": batch.proposals.len() }),
        )?;
        let mut response = format!(
            "Generated {} category proposals in {} ms (batch {}).",
            batch.proposals.len(),
            duration_ms,
            batch.batch_id
        );
        for proposal in batch
            .proposals
            .iter()
            .take(categorization_cfg.max_proposals as usize)
        {
            response.push_str(&format!(
                "\n- {} ({} papers, confidence {:.2})",
                proposal.definition.name,
                proposal.member_entry_ids.len(),
                proposal.definition.confidence.unwrap_or(0.0)
            ));
        }
        Ok(response)
    }

    pub fn categories_apply(
        &mut self,
        mut accepted_ids: Vec<Uuid>,
        renames: HashMap<Uuid, String>,
        rejected_ids: Vec<Uuid>,
    ) -> Result<String> {
        let base = self.active_base()?;
        let store = CategoryProposalStore::new(&base)?;
        let batch = store
            .latest_batch()?
            .context("No proposal batch found. Run `categories propose` first.")?;

        for id in renames.keys() {
            if !accepted_ids.contains(id) {
                accepted_ids.push(*id);
            }
        }
        if accepted_ids.is_empty() {
            anyhow::bail!("Provide at least one proposal id to apply or rename.");
        }

        let mut proposal_map: HashMap<Uuid, _> = batch
            .proposals
            .iter()
            .map(|p| (p.proposal_id, p.clone()))
            .collect();
        let mut applied = Vec::new();
        for proposal_id in accepted_ids {
            let mut proposal = proposal_map
                .remove(&proposal_id)
                .with_context(|| format!("Proposal {} not found in latest batch.", proposal_id))?;
            if let Some(new_name) = renames.get(&proposal_id) {
                proposal.definition.name = new_name.clone();
                proposal.definition.slug = category_slug(new_name);
            }
            proposal.definition.origin = CategoryOrigin::Proposed;
            proposal.definition.updated_at = Utc::now();
            applied.push(proposal);
        }
        if applied.is_empty() {
            anyhow::bail!("No matching proposals were applied.");
        }

        let category_store = CategoryStore::new(&base)?;
        let assignments_index = CategoryAssignmentsIndex::new(&base)?;
        for proposal in &applied {
            let record =
                CategoryRecord::new(proposal.definition.clone(), proposal.narrative.clone());
            category_store.save(&record)?;
            if !proposal.member_entry_ids.is_empty() {
                let assignments: Vec<CategoryAssignment> = proposal
                    .member_entry_ids
                    .iter()
                    .map(|paper_id| CategoryAssignment {
                        assignment_id: Uuid::new_v4(),
                        category_id: proposal.definition.category_id,
                        paper_id: *paper_id,
                        source: AssignmentSource::Auto,
                        confidence: proposal.definition.confidence.unwrap_or(0.6),
                        status: AssignmentStatus::PendingReview,
                        last_reviewed_at: None,
                    })
                    .collect();
                assignments_index
                    .replace_category(&proposal.definition.category_id, &assignments)?;
            }
        }

        let entries = self.manager.load_library_entries(&base)?;
        generate_and_log_reports(&self.manager, &base, &entries)?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_proposals_applied(
            &base,
            CategoryProposalEvent {
                batch_id: batch.batch_id,
                proposed_count: batch.proposals.len(),
                accepted_count: applied.len(),
                rejected_count: rejected_ids.len(),
                duration_ms: batch.duration_ms,
                consent_manifest_id: None,
            },
        )?;

        Ok(format!(
            "Applied {} proposals from batch {}. Reports regenerated.",
            applied.len(),
            batch.batch_id
        ))
    }

    pub fn category_rename(&mut self, current_name: &str, new_name: &str) -> Result<String> {
        let base = self.active_base()?;
        let store = CategoryStore::new(&base)?;
        let assignments = CategoryAssignmentsIndex::new(&base)?;
        let snapshot = CategorySnapshotStore::new(&base)?;
        snapshot.capture(&store, &assignments, "category_rename")?;
        let record = store
            .find_by_name(current_name)?
            .with_context(|| format!("Category '{}' not found", current_name))?;
        if store.name_exists(new_name, Some(&record.definition.category_id))? {
            anyhow::bail!("A category named '{}' already exists.", new_name);
        }
        let updated = store.rename(&record.definition.category_id, new_name)?;
        let entries = self.manager.load_library_entries(&base)?;
        generate_and_log_reports(&self.manager, &base, &entries)?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_edit(
            &base,
            CategoryEditEventDetails {
                edit_type: CategoryEditType::Rename,
                category_ids: vec![updated.definition.category_id],
                snapshot_id: None,
                details: serde_json::json!({ "from": current_name, "to": new_name }),
            },
        )?;
        Ok(format!(
            "Renamed category '{}' to '{}'.",
            current_name, new_name
        ))
    }

    pub fn category_merge(
        &mut self,
        names: Vec<String>,
        target_name: &str,
        keep_pinned: bool,
    ) -> Result<String> {
        let base = self.active_base()?;
        let store = CategoryStore::new(&base)?;
        let assignments = CategoryAssignmentsIndex::new(&base)?;
        let snapshot = CategorySnapshotStore::new(&base)?;
        snapshot.capture(&store, &assignments, "category_merge")?;
        let mut ids = Vec::new();
        for name in &names {
            let record = store
                .find_by_name(name)?
                .with_context(|| format!("Category '{}' not found", name))?;
            ids.push(record.definition.category_id);
        }
        if store.name_exists(target_name, None)? {
            let existing = store
                .find_by_name(target_name)?
                .map(|rec| rec.definition.category_id);
            if existing.map(|id| !ids.contains(&id)).unwrap_or(true) {
                anyhow::bail!(
                    "Target category name '{}' is already used by another category.",
                    target_name
                );
            }
        }
        let outcome = merge_categories(
            &store,
            &assignments,
            MergeOptions {
                source_ids: ids.clone(),
                target_name: target_name.to_string(),
                keep_pinned,
            },
        )?;
        let entries = self.manager.load_library_entries(&base)?;
        generate_and_log_reports(&self.manager, &base, &entries)?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_edit(
            &base,
            CategoryEditEventDetails {
                edit_type: CategoryEditType::Merge,
                category_ids: vec![outcome.merged_category.definition.category_id],
                snapshot_id: None,
                details: serde_json::json!({
                    "merged": names,
                    "target": target_name,
                    "keep_pinned": keep_pinned
                }),
            },
        )?;
        Ok(format!(
            "Merged {} categories into '{}'. Reports updated.",
            outcome.merged_ids.len(),
            target_name
        ))
    }

    pub fn category_move(
        &mut self,
        paper_ids: Vec<Uuid>,
        target_category: &str,
        remove_from_previous: bool,
    ) -> Result<String> {
        if paper_ids.is_empty() {
            anyhow::bail!("Provide at least one paper id to move.");
        }
        let base = self.active_base()?;
        let store = CategoryStore::new(&base)?;
        let assignments = CategoryAssignmentsIndex::new(&base)?;
        let snapshot = CategorySnapshotStore::new(&base)?;
        snapshot.capture(&store, &assignments, "category_move")?;
        let target = store
            .find_by_name(target_category)?
            .with_context(|| format!("Target category '{}' not found", target_category))?;
        move_papers(
            &assignments,
            &target.definition.category_id,
            &paper_ids,
            remove_from_previous,
        )?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_edit(
            &base,
            CategoryEditEventDetails {
                edit_type: CategoryEditType::Move,
                category_ids: vec![target.definition.category_id],
                snapshot_id: None,
                details: serde_json::json!({
                    "paper_count": paper_ids.len(),
                    "target": target_category,
                    "remove_from_previous": remove_from_previous
                }),
            },
        )?;
        Ok(format!(
            "Moved {} papers into '{}'.",
            paper_ids.len(),
            target_category
        ))
    }

    pub fn category_split(&mut self, category_name: &str, rule: &str) -> Result<String> {
        let base = self.active_base()?;
        let store = CategoryStore::new(&base)?;
        let assignments = CategoryAssignmentsIndex::new(&base)?;
        let snapshot = CategorySnapshotStore::new(&base)?;
        snapshot.capture(&store, &assignments, "category_split")?;
        let record = store
            .find_by_name(category_name)?
            .with_context(|| format!("Category '{}' not found", category_name))?;
        let entry_map = self
            .manager
            .load_library_entries(&base)?
            .into_iter()
            .map(|entry| (entry.entry_id, entry))
            .collect::<HashMap<_, _>>();
        let assignment_ids = assignments.list_for_category(&record.definition.category_id)?;
        let mut assigned_entries = Vec::new();
        for assignment in assignment_ids {
            if let Some(entry) = entry_map.get(&assignment.paper_id) {
                assigned_entries.push(entry.clone());
            }
        }
        if assigned_entries.len() < 2 {
            anyhow::bail!("Not enough papers to split this category.");
        }
        let suggestion = split::suggest_split(&record, &assigned_entries, rule);
        store.delete(&record.definition.category_id)?;
        for child in &suggestion.children {
            store.save(&child.record)?;
            let child_assignments: Vec<CategoryAssignment> = child
                .paper_ids
                .iter()
                .map(|paper_id| CategoryAssignment {
                    assignment_id: Uuid::new_v4(),
                    category_id: child.record.definition.category_id,
                    paper_id: *paper_id,
                    source: AssignmentSource::Auto,
                    confidence: 0.7,
                    status: AssignmentStatus::PendingReview,
                    last_reviewed_at: None,
                })
                .collect();
            assignments
                .replace_category(&child.record.definition.category_id, &child_assignments)?;
        }
        let entries = self.manager.load_library_entries(&base)?;
        generate_and_log_reports(&self.manager, &base, &entries)?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_edit(
            &base,
            CategoryEditEventDetails {
                edit_type: CategoryEditType::Split,
                category_ids: suggestion
                    .children
                    .iter()
                    .map(|child| child.record.definition.category_id)
                    .collect(),
                snapshot_id: None,
                details: serde_json::json!({ "parent": category_name, "rule": rule }),
            },
        )?;
        Ok(format!(
            "Split '{}' into {} child categories.",
            category_name,
            suggestion.children.len()
        ))
    }

    pub fn category_undo(&mut self) -> Result<String> {
        let base = self.active_base()?;
        let store = CategoryStore::new(&base)?;
        let assignments = CategoryAssignmentsIndex::new(&base)?;
        let snapshot = CategorySnapshotStore::new(&base)?;
        let latest = snapshot
            .list()?
            .into_iter()
            .next()
            .with_context(|| "No category snapshot available to undo.")?;
        let started = Instant::now();
        snapshot.restore(&latest.snapshot_id, &store, &assignments)?;
        let entries = self.manager.load_library_entries(&base)?;
        generate_and_log_reports(&self.manager, &base, &entries)?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_edit(
            &base,
            CategoryEditEventDetails {
                edit_type: CategoryEditType::Undo,
                category_ids: Vec::new(),
                snapshot_id: Some(latest.snapshot_id),
                details: serde_json::json!({ "reason": latest.reason }),
            },
        )?;
        log.record_category_operation_metrics(
            "category_undo",
            started.elapsed().as_millis() as i64,
            true,
            serde_json::json!({ "snapshot_id": latest.snapshot_id }),
        )?;
        Ok("Restored previous category snapshot.".into())
    }

    pub fn category_narrative(
        &mut self,
        category_name: &str,
        summary: Option<String>,
        learning_prompts: Option<Vec<String>>,
        notes: Option<Vec<String>>,
        pinned_papers: Option<Vec<Uuid>>,
        figure_gallery_enabled: Option<bool>,
        ai_assist_approval: Option<&str>,
    ) -> Result<String> {
        let base = self.active_base()?;
        if let Some(approval) = ai_assist_approval {
            if approval.trim().is_empty() {
                anyhow::bail!("AI assistance requires non-empty approval text.");
            }
            let scope = ConsentScope {
                batch_id: None,
                paper_ids: pinned_papers.clone().unwrap_or_default(),
            };
            require_remote_operation_consent(
                &self.manager,
                &base,
                ConsentOperation::CategoryNarrativeSuggest,
                approval,
                scope,
                serde_json::json!({ "category": category_name }),
            )?;
        }
        let store = CategoryStore::new(&base)?;
        let assignments = CategoryAssignmentsIndex::new(&base)?;
        let snapshot = CategorySnapshotStore::new(&base)?;
        snapshot.capture(&store, &assignments, "category_narrative")?;
        let record = store
            .find_by_name(category_name)?
            .with_context(|| format!("Category '{}' not found", category_name))?;
        let update = NarrativeUpdate {
            summary,
            learning_prompts,
            notes,
            pinned_papers,
            figure_gallery_enabled,
        };
        let updated = apply_narrative_update(&store, &record.definition.category_id, update)?;
        let entries = self.manager.load_library_entries(&base)?;
        generate_and_log_reports(&self.manager, &base, &entries)?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_edit(
            &base,
            CategoryEditEventDetails {
                edit_type: CategoryEditType::NarrativeEdit,
                category_ids: vec![updated.definition.category_id],
                snapshot_id: None,
                details: serde_json::json!({
                    "category": category_name,
                    "pinned_count": updated.definition.pinned_papers.len(),
                    "figure_gallery_enabled": updated.definition.figure_gallery_enabled
                }),
            },
        )?;
        Ok(format!(
            "Updated narrative for '{}' and regenerated reports.",
            category_name
        ))
    }

    pub fn category_pin(
        &mut self,
        category_name: &str,
        paper_id: Uuid,
        pin: bool,
    ) -> Result<String> {
        let base = self.active_base()?;
        let entries = self.manager.load_library_entries(&base)?;
        if !entries.iter().any(|entry| entry.entry_id == paper_id) {
            anyhow::bail!("Paper {} does not exist in this Base.", paper_id);
        }
        let store = CategoryStore::new(&base)?;
        let assignments = CategoryAssignmentsIndex::new(&base)?;
        let snapshot = CategorySnapshotStore::new(&base)?;
        snapshot.capture(&store, &assignments, "category_pin")?;
        let record = store
            .find_by_name(category_name)?
            .with_context(|| format!("Category '{}' not found", category_name))?;
        let mut pinned = record.definition.pinned_papers.clone();
        if pin {
            if !pinned.contains(&paper_id) {
                pinned.push(paper_id);
            }
        } else {
            pinned.retain(|existing| existing != &paper_id);
        }
        let updated = apply_narrative_update(
            &store,
            &record.definition.category_id,
            NarrativeUpdate {
                pinned_papers: Some(pinned.clone()),
                ..Default::default()
            },
        )?;
        generate_and_log_reports(&self.manager, &base, &entries)?;
        let log = OrchestrationLog::for_base(&base);
        log.log_category_edit(
            &base,
            CategoryEditEventDetails {
                edit_type: CategoryEditType::PinToggle,
                category_ids: vec![updated.definition.category_id],
                snapshot_id: None,
                details: serde_json::json!({
                    "category": category_name,
                    "paper_id": paper_id,
                    "pin": pin
                }),
            },
        )?;
        let verb = if pin { "Pinned" } else { "Unpinned" };
        Ok(format!(
            "{} paper {} for '{}'.",
            verb, paper_id, category_name
        ))
    }

    pub fn categories_status(&mut self, include_backlog: bool) -> Result<String> {
        let base = self.active_base()?;
        let entries = self.manager.load_library_entries(&base)?;
        let store = CategoryStore::new(&base)?;
        let categories = store.list()?;
        let assignments_index = CategoryAssignmentsIndex::new(&base)?;
        let assignments = assignments_index.list_all()?;
        let summary =
            CategoryMetricsCollector::collect(&entries, &categories, &assignments, include_backlog);
        let metrics_store = CategoryMetricsStore::new(&base);
        metrics_store.save(&summary.metrics)?;
        let mut response = format!(
            "Category status ({} categories, {} assignments):",
            categories.len(),
            assignments.len()
        );
        let name_map: HashMap<Uuid, String> = categories
            .iter()
            .map(|record| {
                (
                    record.definition.category_id,
                    record.definition.name.clone(),
                )
            })
            .collect();
        let mut alerts = Vec::new();
        for metric in &summary.metrics {
            if let Some(category_id) = metric.category_id {
                if metric.overload_ratio > 0.25 || metric.staleness_days > 30 {
                    let name = name_map
                        .get(&category_id)
                        .cloned()
                        .unwrap_or_else(|| category_id.to_string());
                    alerts.push(format!(
                        "{}: {} papers, staleness {}d, {:.0}% of library",
                        name,
                        metric.paper_count,
                        metric.staleness_days,
                        metric.overload_ratio * 100.0
                    ));
                }
            }
        }
        if alerts.is_empty() {
            response.push_str("\n- No overload or staleness alerts.");
        } else {
            response.push_str("\n- Alerts:\n");
            for alert in alerts {
                response.push_str(&format!("  * {}\n", alert));
            }
        }
        if include_backlog {
            if summary.backlog_segments.is_empty() {
                response.push_str("\n- Backlog clear.");
            } else {
                response.push_str("\n- Backlog segments:\n");
                for segment in summary.backlog_segments.iter().take(3) {
                    response.push_str(&format!(
                        "  * {} ({} uncategorized)\n",
                        segment.label, segment.count
                    ));
                }
            }
        }
        Ok(response.trim().to_string())
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
