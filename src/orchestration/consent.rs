use crate::bases::{Base, BaseManager};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentManifest {
    pub manifest_id: Uuid,
    pub base_id: Uuid,
    pub operation: ConsentOperation,
    pub scope: ConsentScope,
    pub approval_text: String,
    pub approved_at: DateTime<Utc>,
    pub prompt_manifest: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentScope {
    pub batch_id: Option<Uuid>,
    pub paper_ids: Vec<Uuid>,
}

impl Default for ConsentScope {
    fn default() -> Self {
        Self {
            batch_id: None,
            paper_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentOperation {
    MetadataLookup,
    FigureExtraction,
}

/// File-backed store for consent manifests scoped to a Base.
pub struct ConsentStore {
    path: PathBuf,
}

impl ConsentStore {
    pub fn for_base(base: &Base) -> Self {
        let path = base.ai_layer_path.join("consent_manifests.jsonl");
        Self { path }
    }

    pub fn record(&self, manifest: &ConsentManifest) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(serde_json::to_string(manifest)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn latest_for_operation(
        &self,
        operation: ConsentOperation,
    ) -> Result<Option<ConsentManifest>> {
        if !self.path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&self.path)?;
        for line in data.lines().filter(|l| !l.trim().is_empty()).rev() {
            let manifest: ConsentManifest = serde_json::from_str(line)?;
            if manifest.operation == operation {
                return Ok(Some(manifest));
            }
        }
        Ok(None)
    }
}

/// Convenience helper that validates remote permissions before requiring consent.
pub fn require_remote_operation_consent(
    manager: &BaseManager,
    base: &Base,
    operation: ConsentOperation,
    approval_text: &str,
    scope: ConsentScope,
    prompt_manifest: serde_json::Value,
) -> Result<ConsentManifest> {
    match operation {
        ConsentOperation::MetadataLookup => {
            if !manager.config.ingestion.remote_metadata_allowed
                && !manager.config.acquisition.remote_allowed
            {
                anyhow::bail!(
                    "Remote metadata lookups are disabled for this install. Enable them in config before proceeding."
                );
            }
        }
        ConsentOperation::FigureExtraction => {
            if !manager.config.acquisition.remote_allowed {
                anyhow::bail!(
                    "Remote figure extraction is disabled for this install. Enable it in config to continue."
                );
            }
        }
    }

    let manifest = ConsentManifest {
        manifest_id: Uuid::new_v4(),
        base_id: base.id,
        operation,
        scope,
        approval_text: approval_text.to_string(),
        approved_at: Utc::now(),
        prompt_manifest,
    };
    let store = ConsentStore::for_base(base);
    store.record(&manifest)?;
    Ok(manifest)
}
