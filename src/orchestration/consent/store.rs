use crate::bases::{Base, BaseManager, ProfileLayout};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
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
    #[serde(default)]
    pub prompt_excerpt: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub data_categories: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: ConsentStatus,
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
pub enum ConsentStatus {
    Approved,
    Rejected,
    Revoked,
}

impl Default for ConsentStatus {
    fn default() -> Self {
        Self::Approved
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentOperation {
    MetadataLookup,
    FigureExtraction,
    CategoryNarrativeSuggest,
    VisualizationRemoteLayout,
}

/// File-backed store for consent manifests scoped to a Base.
pub struct ConsentStore {
    root: PathBuf,
}

impl ConsentStore {
    pub fn for_base(base: &Base) -> Self {
        let layout = ProfileLayout::new(base);
        Self {
            root: layout.consent_manifests_dir,
        }
    }

    /// Writes the manifest to disk and returns the file path.
    pub fn record(&self, manifest: &ConsentManifest) -> Result<PathBuf> {
        fs::create_dir_all(&self.root)?;
        let path = self.root.join(format!("{}.json", manifest.manifest_id));
        let data = serde_json::to_vec_pretty(manifest)?;
        fs::write(&path, data)?;
        Ok(path)
    }

    pub fn get(&self, manifest_id: &Uuid) -> Result<Option<ConsentManifest>> {
        let path = self.root.join(format!("{}.json", manifest_id));
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read(&path)?;
        let manifest = serde_json::from_slice(&data)?;
        Ok(Some(manifest))
    }

    pub fn latest_for_operation(
        &self,
        operation: ConsentOperation,
    ) -> Result<Option<ConsentManifest>> {
        let mut manifests = self.load_all()?;
        manifests.retain(|m| m.operation == operation);
        manifests.sort_by_key(|m| m.approved_at);
        Ok(manifests.pop())
    }

    pub fn load_all(&self) -> Result<Vec<ConsentManifest>> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut manifests = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let manifest = read_manifest(entry.path())?;
            manifests.push(manifest);
        }
        Ok(manifests)
    }
}

fn read_manifest(path: PathBuf) -> Result<ConsentManifest> {
    let data = fs::read(&path)
        .with_context(|| format!("Failed reading consent manifest {:?}", path))?;
    let manifest = serde_json::from_slice(&data)
        .with_context(|| format!("Failed parsing consent manifest {:?}", path))?;
    Ok(manifest)
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
        ConsentOperation::CategoryNarrativeSuggest => {
            if !manager.config.acquisition.remote_allowed {
                anyhow::bail!(
                    "Remote narrative assistance is disabled for this install. Enable remote access before requesting AI summaries."
                );
            }
        }
        ConsentOperation::VisualizationRemoteLayout => {
            if !manager.config.acquisition.remote_allowed {
                anyhow::bail!(
                    "Remote visualization layouts are disabled for this install. Enable remote access before continuing."
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
        prompt_excerpt: None,
        provider: None,
        data_categories: Vec::new(),
        expires_at: None,
        status: ConsentStatus::Approved,
    };
    let store = ConsentStore::for_base(base);
    store.record(&manifest)?;
    Ok(manifest)
}
