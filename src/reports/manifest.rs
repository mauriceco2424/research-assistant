use crate::bases::Base;
use crate::reports::config_store::ReportConfigOverrides;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const MANIFEST_DIR: &str = "reports/manifests";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportScopeMode {
    All,
    GlobalOnly,
    Categories,
    Includes,
}

impl ReportScopeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportScopeMode::All => "all",
            ReportScopeMode::GlobalOnly => "global_only",
            ReportScopeMode::Categories => "categories",
            ReportScopeMode::Includes => "includes",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScope {
    pub mode: String,
    #[serde(default)]
    pub categories: Vec<Uuid>,
    #[serde(default)]
    pub includes: Vec<String>,
}

impl Default for ReportScope {
    fn default() -> Self {
        Self {
            mode: ReportScopeMode::All.as_str().into(),
            categories: Vec::new(),
            includes: Vec::new(),
        }
    }
}

impl ReportScope {
    pub fn mode(&self) -> ReportScopeMode {
        match self.mode.as_str() {
            "global_only" => ReportScopeMode::GlobalOnly,
            "categories" => ReportScopeMode::Categories,
            "includes" => ReportScopeMode::Includes,
            _ => ReportScopeMode::All,
        }
    }

    pub fn label(&self) -> String {
        match self.mode() {
            ReportScopeMode::All => "all reports".into(),
            ReportScopeMode::GlobalOnly => "global summary".into(),
            ReportScopeMode::Categories => {
                if self.categories.is_empty() {
                    "selected categories".into()
                } else {
                    format!("{} categories", self.categories.len())
                }
            }
            ReportScopeMode::Includes => {
                if self.includes.is_empty() {
                    "custom selection".into()
                } else {
                    format!("{} outputs", self.includes.len())
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportBuildRequest {
    pub request_id: Uuid,
    pub base_id: Uuid,
    pub scope: ReportScope,
    pub overrides: ReportConfigOverrides,
    pub requested_at: DateTime<Utc>,
    #[serde(default)]
    pub overwrite_existing: bool,
}

impl ReportBuildRequest {
    pub fn new(
        base_id: Uuid,
        scope: ReportScope,
        overrides: ReportConfigOverrides,
        overwrite_existing: bool,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            base_id,
            scope,
            overrides,
            requested_at: Utc::now(),
            overwrite_existing,
        }
    }

    pub fn scope_label(&self) -> String {
        self.scope.label()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportOutputEntry {
    pub path: PathBuf,
    pub scope: String,
    pub hash: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportManifest {
    pub manifest_id: Uuid,
    pub build_request_id: Uuid,
    pub base_id: Uuid,
    pub ai_layer_snapshots: Vec<Uuid>,
    pub metrics_revision_id: Option<Uuid>,
    pub visualization_dataset_ids: Vec<Uuid>,
    pub config_signature: String,
    pub consent_tokens: Vec<Uuid>,
    pub outputs: Vec<ReportOutputEntry>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub orchestration_id: Option<Uuid>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ReportManifest {
    pub fn new(base: &Base, build_request_id: Uuid, config_signature: String) -> Self {
        Self {
            manifest_id: Uuid::new_v4(),
            build_request_id,
            base_id: base.id,
            config_signature,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            ..Default::default()
        }
    }

    pub fn add_output(&mut self, entry: ReportOutputEntry) {
        self.outputs.push(entry);
    }

    pub fn persist(&self, base: &Base) -> Result<PathBuf> {
        let dir = base.ai_layer_path.join(MANIFEST_DIR);
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
        let path = dir.join(format!("{}.json", self.manifest_id));
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShareBundleDescriptor {
    pub bundle_id: Uuid,
    pub manifest_id: Uuid,
    pub destination: PathBuf,
    pub format: String,
    pub include_visualizations: bool,
    pub include_figures: bool,
    pub checksum: Option<String>,
    pub size_bytes: Option<u64>,
}

pub fn read_manifest(path: &Path) -> Result<ReportManifest> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("Missing manifest {}", path.display()))?;
    let manifest = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid manifest {}", path.display()))?;
    Ok(manifest)
}

pub fn hash_path(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Unable to open {} for hashing", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
