use crate::bases::{Base, BaseManager};
use crate::orchestration::{
    require_remote_operation_consent, ConsentManifest as OrchestrationConsentManifest,
    ConsentOperation, ConsentScope,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const CONSENT_LOG: &str = "reports/consent_log.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentManifest {
    pub consent_id: Uuid,
    pub operation: String,
    pub data_categories: Vec<String>,
    pub endpoint: Option<String>,
    pub approved_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct ConsentRegistry<'a> {
    manager: &'a BaseManager,
    base: Base,
    path: PathBuf,
}

impl<'a> ConsentRegistry<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        let path = base.ai_layer_path.join(CONSENT_LOG);
        Self {
            manager,
            base: base.clone(),
            path,
        }
    }

    pub fn record(&self, manifest: ConsentManifest) -> Result<()> {
        let mut manifests = self.load_all()?;
        manifests.push(manifest);
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let data = serde_json::to_string_pretty(&manifests)?;
        fs::write(&self.path, data)?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<ConsentManifest>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn ensure_remote_consent(
        &self,
        consent_op: ConsentOperation,
        scope: ConsentScope,
        approval_text: &str,
        prompt_manifest: Value,
    ) -> Result<OrchestrationConsentManifest> {
        require_remote_operation_consent(
            self.manager,
            &self.base,
            consent_op,
            approval_text,
            scope,
            prompt_manifest,
        )
    }
}
