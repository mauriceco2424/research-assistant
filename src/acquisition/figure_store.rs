use crate::bases::Base;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureAssetRecord {
    pub asset_id: Uuid,
    pub paper_id: Uuid,
    pub caption: String,
    pub image_path: PathBuf,
    pub approval_batch_id: Option<Uuid>,
    pub extraction_status: FigureExtractionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FigureExtractionStatus {
    Pending,
    Success,
    Failed,
    Manual,
}

pub struct FigureStore {
    metadata_path: PathBuf,
    assets_dir: PathBuf,
}

impl FigureStore {
    pub fn new(base: &Base) -> Self {
        let metadata_path = base.ai_layer_path.join("figure_assets.jsonl");
        let assets_dir = base.user_layer_path.join("figures");
        Self {
            metadata_path,
            assets_dir,
        }
    }

    pub fn assets_dir(&self) -> &PathBuf {
        &self.assets_dir
    }

    pub fn store_image(&self, batch_id: &Uuid, file_name: &str, bytes: &[u8]) -> Result<PathBuf> {
        fs::create_dir_all(&self.assets_dir)?;
        let batch_dir = self.assets_dir.join(batch_id.to_string());
        fs::create_dir_all(&batch_dir)?;
        let path = batch_dir.join(file_name);
        fs::write(&path, bytes)?;
        Ok(path)
    }

    pub fn append_record(&self, record: &FigureAssetRecord) -> Result<()> {
        if let Some(parent) = self.metadata_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.metadata_path)?;
        file.write_all(serde_json::to_string(record)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn load_records(&self) -> Result<Vec<FigureAssetRecord>> {
        if !self.metadata_path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.metadata_path)?;
        let mut records = Vec::new();
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            let record: FigureAssetRecord = serde_json::from_str(line)?;
            records.push(record);
        }
        Ok(records)
    }

    pub fn remove_records_for_batch(&self, batch_id: &Uuid) -> Result<Vec<FigureAssetRecord>> {
        let mut records = self.load_records()?;
        let mut removed = Vec::new();
        records.retain(|record| {
            if record.approval_batch_id == Some(*batch_id) {
                if record.image_path.exists() {
                    let _ = fs::remove_file(&record.image_path);
                }
                removed.push(record.clone());
                false
            } else {
                true
            }
        });
        self.persist_all(&records)?;
        Ok(removed)
    }

    pub fn persist_all(&self, records: &[FigureAssetRecord]) -> Result<()> {
        if let Some(parent) = self.metadata_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&self.metadata_path)?;
        for record in records {
            file.write_all(serde_json::to_string(record)?.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }
}
