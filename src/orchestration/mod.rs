pub mod consent;

pub use consent::{
    require_remote_operation_consent, ConsentManifest, ConsentOperation, ConsentScope, ConsentStore,
};

use crate::bases::{Base, BaseManager};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// Type of orchestration events that can be logged.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl OrchestrationLog {
    pub fn for_base(base: &Base) -> Self {
        let events_path = base.ai_layer_path.join("events.jsonl");
        let batches_path = base.ai_layer_path.join("acquisition_batches.jsonl");
        let figure_batches_path = base.ai_layer_path.join("figure_batches.jsonl");
        Self {
            events_path,
            batches_path,
            figure_batches_path,
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
