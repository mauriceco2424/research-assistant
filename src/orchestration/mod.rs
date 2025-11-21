pub mod consent;
pub mod events;
pub mod intent;
pub mod learning;
pub mod profiles;
pub mod report_progress;

pub use consent::{
    require_remote_operation_consent, ConsentManifest, ConsentOperation, ConsentScope, ConsentStore,
};
pub use events::{log_profile_event, log_profile_event_with_id, ProfileEventDetails};
pub use learning::interface::LearningInterface;
pub use report_progress::ReportProgressTracker;

use crate::bases::{Base, BaseManager};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// Maximum acceptable ingestion duration before raising an SLA warning (seconds).
pub const INGESTION_SLA_SECS: i64 = 5 * 60;
/// Maximum acceptable report regeneration duration before raising an SLA warning (seconds).
pub const REPORT_SLA_SECS: i64 = 60;

/// Type of orchestration events that can be logged.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    BaseCreated,
    BaseSelected,
    PathAIngestion,
    PathBInterview,
    AcquisitionApproved,
    AcquisitionUndo,
    IngestionBatchStarted,
    IngestionBatchPaused,
    IngestionBatchResumed,
    IngestionBatchCompleted,
    FigureExtractionRequested,
    FigureExtractionCompleted,
    FigureExtractionUndo,
    MetadataRefreshRequested,
    MetadataRefreshApplied,
    MetadataRefreshUndo,
    ReportsGenerated,
    ReportsShared,
    CategoryProposalsGenerated,
    CategoryProposalsApplied,
    CategoryEdit,
    #[serde(rename = "project_created")]
    WritingProjectCreated,
    #[serde(rename = "style_model_ingested")]
    WritingStyleModelIngested,
    #[serde(rename = "outline_created")]
    WritingOutlineCreated,
    #[serde(rename = "outline_modified")]
    WritingOutlineModified,
    #[serde(rename = "draft_generated")]
    WritingDraftGenerated,
    #[serde(rename = "section_edited")]
    WritingSectionEdited,
    #[serde(rename = "citation_flagged")]
    WritingCitationFlagged,
    #[serde(rename = "compile_attempted")]
    WritingCompileAttempted,
    #[serde(rename = "undo_applied")]
    WritingUndoApplied,
    LearningSessionStarted,
    LearningQuestionGenerated,
    LearningAnswerEvaluated,
    LearningKnowledgeUpdated,
    LearningUndoApplied,
    ProfileChange,
    IntentDetected,
    IntentConfirmed,
    IntentExecuted,
    IntentFailed,
}

/// General-purpose orchestration event stored as JSONL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationEvent {
    pub event_id: Uuid,
    pub base_id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
}

/// Record of a single acquisition batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionBatch {
    pub batch_id: Uuid,
    pub base_id: Uuid,
    pub approved_text: String,
    pub requested_at: DateTime<Utc>,
    pub approved_at: DateTime<Utc>,
    pub records: Vec<AcquisitionRecord>,
}

impl AcquisitionBatch {
    pub fn new(
        base_id: Uuid,
        approved_text: String,
        records: Vec<AcquisitionRecord>,
        requested_at: DateTime<Utc>,
    ) -> Self {
        Self {
            batch_id: Uuid::new_v4(),
            base_id,
            approved_text,
            requested_at,
            approved_at: Utc::now(),
            records,
        }
    }

    pub fn created_entry_ids(&self) -> Vec<Uuid> {
        self.records
            .iter()
            .filter_map(|r| r.library_entry_id)
            .collect()
    }
}

/// Information about a single candidate inside a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionRecord {
    pub candidate_identifier: String,
    pub title: String,
    pub authors: Vec<String>,
    pub pdf_attached: bool,
    pub needs_pdf: bool,
    pub library_entry_id: Option<Uuid>,
}

/// Wraps log paths for a base.
pub struct OrchestrationLog {
    events_path: PathBuf,
    batches_path: PathBuf,
    figure_batches_path: PathBuf,
    metrics_path: PathBuf,
}

impl OrchestrationLog {
    pub fn for_base(base: &Base) -> Self {
        let events_path = base.ai_layer_path.join("events.jsonl");
        let batches_path = base.ai_layer_path.join("acquisition_batches.jsonl");
        let figure_batches_path = base.ai_layer_path.join("figure_batches.jsonl");
        let metrics_path = base.ai_layer_path.join("metrics.jsonl");
        Self {
            events_path,
            batches_path,
            figure_batches_path,
            metrics_path,
        }
    }

    pub fn append_event(&self, event: &OrchestrationEvent) -> Result<()> {
        if let Some(parent) = self.events_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)?;
        file.write_all(serde_json::to_string(event)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn record_batch(&self, batch: &AcquisitionBatch) -> Result<()> {
        if let Some(parent) = self.batches_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.batches_path)?;
        file.write_all(serde_json::to_string(batch)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn load_batches(&self) -> Result<Vec<AcquisitionBatch>> {
        if !self.batches_path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.batches_path)?;
        let mut batches = Vec::new();
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            let batch: AcquisitionBatch = serde_json::from_str(line)?;
            batches.push(batch);
        }
        Ok(batches)
    }

    pub fn undo_last_batch(
        &self,
        base: &Base,
        manager: &BaseManager,
    ) -> Result<Option<AcquisitionBatch>> {
        let mut batches = self.load_batches()?;
        if let Some(batch) = batches.pop() {
            // Persist updated batch history
            self.persist_batches(&batches)?;
            // Remove library entries created by the batch
            let ids = batch.created_entry_ids();
            if !ids.is_empty() {
                manager.remove_entries_by_ids(base, &ids)?;
            }
            // Append an undo event
            let event = OrchestrationEvent {
                event_id: Uuid::new_v4(),
                base_id: base.id,
                event_type: EventType::AcquisitionUndo,
                timestamp: Utc::now(),
                details: serde_json::json!({ "batch_id": batch.batch_id }),
            };
            self.append_event(&event)?;
            Ok(Some(batch))
        } else {
            Ok(None)
        }
    }

    fn persist_batches(&self, batches: &[AcquisitionBatch]) -> Result<()> {
        if let Some(parent) = self.batches_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&self.batches_path)?;
        for batch in batches {
            file.write_all(serde_json::to_string(batch)?.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn record_figure_batch(&self, batch: &FigureExtractionBatch) -> Result<()> {
        if let Some(parent) = self.figure_batches_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.figure_batches_path)?;
        file.write_all(serde_json::to_string(batch)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn load_figure_batches(&self) -> Result<Vec<FigureExtractionBatch>> {
        if !self.figure_batches_path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.figure_batches_path)?;
        let mut batches = Vec::new();
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            let batch: FigureExtractionBatch = serde_json::from_str(line)?;
            batches.push(batch);
        }
        Ok(batches)
    }

    pub fn undo_last_figure_batch(&self) -> Result<Option<FigureExtractionBatch>> {
        let mut batches = self.load_figure_batches()?;
        if let Some(batch) = batches.pop() {
            self.persist_figure_batches(&batches)?;
            let event = OrchestrationEvent {
                event_id: Uuid::new_v4(),
                base_id: batch.base_id,
                event_type: EventType::FigureExtractionUndo,
                timestamp: Utc::now(),
                details: serde_json::json!({ "batch_id": batch.batch_id }),
            };
            self.append_event(&event)?;
            Ok(Some(batch))
        } else {
            Ok(None)
        }
    }

    fn persist_figure_batches(&self, batches: &[FigureExtractionBatch]) -> Result<()> {
        if let Some(parent) = self.figure_batches_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&self.figure_batches_path)?;
        for batch in batches {
            file.write_all(serde_json::to_string(batch)?.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn load_events(&self) -> Result<Vec<OrchestrationEvent>> {
        if !self.events_path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.events_path)?;
        let mut events = Vec::new();
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            let event: OrchestrationEvent = serde_json::from_str(line)?;
            events.push(event);
        }
        Ok(events)
    }

    pub fn load_events_since(&self, cutoff: DateTime<Utc>) -> Result<Vec<OrchestrationEvent>> {
        Ok(self
            .load_events()?
            .into_iter()
            .filter(|event| event.timestamp >= cutoff)
            .collect())
    }

    pub fn record_metric(&self, record: &MetricRecord) -> Result<()> {
        if let Some(parent) = self.metrics_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.metrics_path)?;
        file.write_all(serde_json::to_string(record)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn load_metrics(&self) -> Result<Vec<MetricRecord>> {
        if !self.metrics_path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.metrics_path)?;
        let mut metrics = Vec::new();
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            let record: MetricRecord = serde_json::from_str(line)?;
            metrics.push(record);
        }
        Ok(metrics)
    }

    pub fn record_category_operation_metrics(
        &self,
        operation: &str,
        duration_ms: i64,
        success: bool,
        details: serde_json::Value,
    ) -> Result<()> {
        self.record_metric(&MetricRecord::CategoryOperation(
            CategoryOperationMetricsRecord {
                operation: operation.to_string(),
                duration_ms,
                success,
                details,
            },
        ))
    }

    fn append_structured_event<T: Serialize>(
        &self,
        base: &Base,
        event_type: EventType,
        details: T,
    ) -> Result<()> {
        let event = OrchestrationEvent {
            event_id: Uuid::new_v4(),
            base_id: base.id,
            event_type,
            timestamp: Utc::now(),
            details: serde_json::to_value(details)?,
        };
        self.append_event(&event)
    }

    pub fn log_category_proposals_generated(
        &self,
        base: &Base,
        details: CategoryProposalEvent,
    ) -> Result<()> {
        self.append_structured_event(base, EventType::CategoryProposalsGenerated, details)
    }

    pub fn log_category_proposals_applied(
        &self,
        base: &Base,
        details: CategoryProposalEvent,
    ) -> Result<()> {
        self.append_structured_event(base, EventType::CategoryProposalsApplied, details)
    }

    pub fn log_category_edit(&self, base: &Base, details: CategoryEditEventDetails) -> Result<()> {
        self.append_structured_event(base, EventType::CategoryEdit, details)
    }
}

/// Append a simple orchestration event helper.
pub fn log_event(
    _manager: &BaseManager,
    base: &Base,
    event_type: EventType,
    details: serde_json::Value,
) -> Result<()> {
    let event = OrchestrationEvent {
        event_id: Uuid::new_v4(),
        base_id: base.id,
        event_type,
        timestamp: Utc::now(),
        details,
    };
    let log = OrchestrationLog::for_base(base);
    log.append_event(&event)
}

/// Representation of a figure extraction batch for orchestration logging/undo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureExtractionBatch {
    pub batch_id: Uuid,
    pub base_id: Uuid,
    pub approval_text: String,
    pub requested_at: DateTime<Utc>,
    pub approved_at: DateTime<Utc>,
    pub figure_asset_ids: Vec<Uuid>,
}

impl FigureExtractionBatch {
    pub fn new(
        base_id: Uuid,
        approval_text: String,
        figure_asset_ids: Vec<Uuid>,
        requested_at: DateTime<Utc>,
    ) -> Self {
        Self {
            batch_id: Uuid::new_v4(),
            base_id,
            approval_text,
            requested_at,
            approved_at: Utc::now(),
            figure_asset_ids,
        }
    }
}

/// Metric record emitted by ingestion completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionMetricsRecord {
    pub batch_id: Uuid,
    pub duration_ms: i64,
    pub ingested: u64,
    pub skipped: u64,
    pub failed: u64,
    pub sla_breached: bool,
}

/// Metric record emitted when generating reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetricsRecord {
    pub duration_ms: i64,
    pub entry_count: usize,
    pub figure_count: usize,
    pub sla_breached: bool,
}

/// Metric record covering figure extraction success for transparency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureMetricsRecord {
    pub batch_id: Uuid,
    pub requested: usize,
    pub succeeded: usize,
    pub success_rate: f32,
}

/// Tagged enum persisted as JSONL in `metrics.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "metric", rename_all = "snake_case")]
pub enum MetricRecord {
    Ingestion(IngestionMetricsRecord),
    Reports(ReportMetricsRecord),
    Figure(FigureMetricsRecord),
    CategoryOperation(CategoryOperationMetricsRecord),
    Writing(WritingMetricRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryOperationMetricsRecord {
    pub operation: String,
    pub duration_ms: i64,
    pub success: bool,
    #[serde(default)]
    pub details: serde_json::Value,
}

/// Metrics for writing assistant flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingMetricRecord {
    pub kind: WritingMetricKind,
    pub duration_ms: i64,
    pub success: bool,
    #[serde(default)]
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WritingMetricKind {
    ProjectScaffold,
    OutlineSync,
    CitationResolution,
    Compile,
    Undo,
    Consent,
}

/// Structured payload describing a category proposal batch or acceptance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryProposalEvent {
    pub batch_id: Uuid,
    pub proposed_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub duration_ms: Option<i64>,
    pub consent_manifest_id: Option<Uuid>,
}

/// Structured payload for category edits (rename, merge, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryEditEventDetails {
    pub edit_type: CategoryEditType,
    pub category_ids: Vec<Uuid>,
    pub snapshot_id: Option<Uuid>,
    #[serde(default)]
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategoryEditType {
    ProposeAccept,
    Rename,
    Merge,
    Split,
    Move,
    NarrativeEdit,
    PinToggle,
    Undo,
}
