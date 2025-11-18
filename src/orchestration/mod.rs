use crate::bases::{Base, BaseManager};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
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
}

impl OrchestrationLog {
    pub fn for_base(base: &Base) -> Self {
        let events_path = base.ai_layer_path.join("events.jsonl");
        let batches_path = base.ai_layer_path.join("acquisition_batches.jsonl");
        Self {
            events_path,
            batches_path,
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
}

/// Append a simple orchestration event helper.
pub fn log_event(
    manager: &BaseManager,
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
