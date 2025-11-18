use crate::bases::Base;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// Status lifecycle for ingestion batches.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngestionBatchStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
}

/// JSONL friendly record describing an ingestion batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionBatchState {
    pub batch_id: Uuid,
    pub base_id: Uuid,
    pub source_path: PathBuf,
    pub status: IngestionBatchStatus,
    pub processed_files: u64,
    pub ingested_files: u64,
    pub skipped_files: u64,
    pub failed_files: u64,
    #[serde(default)]
    pub total_files: u64,
    pub last_checkpoint: Option<String>,
    pub remote_metadata_allowed: bool,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IngestionBatchState {
    pub fn new(base: &Base, source_path: PathBuf, remote_metadata_allowed: bool) -> Self {
        Self {
            batch_id: Uuid::new_v4(),
            base_id: base.id,
            source_path,
            status: IngestionBatchStatus::Pending,
            processed_files: 0,
            ingested_files: 0,
            skipped_files: 0,
            failed_files: 0,
            total_files: 0,
            last_checkpoint: None,
            remote_metadata_allowed,
            started_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn mark_running(&mut self) {
        self.status = IngestionBatchStatus::Running;
        self.updated_at = Utc::now();
    }

    pub fn mark_paused(&mut self, checkpoint: Option<String>) {
        self.status = IngestionBatchStatus::Paused;
        self.last_checkpoint = checkpoint;
        self.updated_at = Utc::now();
    }

    pub fn mark_completed(&mut self) {
        self.status = IngestionBatchStatus::Completed;
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self) {
        self.status = IngestionBatchStatus::Failed;
        self.updated_at = Utc::now();
    }

    pub fn update_progress(
        &mut self,
        processed: u64,
        ingested: u64,
        skipped: u64,
        failed: u64,
        checkpoint: Option<String>,
    ) {
        self.processed_files = processed;
        self.ingested_files = ingested;
        self.skipped_files = skipped;
        self.failed_files = failed;
        if checkpoint.is_some() {
            self.last_checkpoint = checkpoint;
        }
        self.updated_at = Utc::now();
    }
}

/// Persists ingestion batches as JSONL in the AI layer.
pub struct IngestionBatchStore {
    path: PathBuf,
}

impl IngestionBatchStore {
    pub fn for_base(base: &Base) -> Self {
        let path = base.ai_layer_path.join("ingestion_batches.jsonl");
        Self { path }
    }

    pub fn list(&self) -> Result<Vec<IngestionBatchState>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.path)
            .with_context(|| format!("Unable to read {:?}", self.path))?;
        let mut batches = Vec::new();
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            let batch: IngestionBatchState = serde_json::from_str(line)
                .with_context(|| "Failed to parse ingestion batch record")?;
            batches.push(batch);
        }
        Ok(batches)
    }

    pub fn latest(&self) -> Result<Option<IngestionBatchState>> {
        Ok(self.list()?.into_iter().max_by_key(|b| b.started_at))
    }

    pub fn get(&self, batch_id: &Uuid) -> Result<Option<IngestionBatchState>> {
        Ok(self.list()?.into_iter().find(|b| &b.batch_id == batch_id))
    }

    pub fn upsert(&self, state: &IngestionBatchState) -> Result<()> {
        let mut batches = self.list()?;
        if let Some(existing) = batches
            .iter_mut()
            .find(|batch| batch.batch_id == state.batch_id)
        {
            *existing = state.clone();
        } else {
            batches.push(state.clone());
        }
        self.persist_all(&batches)
    }

    pub fn append(&self, state: &IngestionBatchState) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(serde_json::to_string(state)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn persist_all(&self, states: &[IngestionBatchState]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&self.path)?;
        for state in states {
            file.write_all(serde_json::to_string(state)?.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }
}
