use crate::bases::{Base, BaseManager};
use crate::orchestration::{require_remote_operation_consent, ConsentOperation, ConsentScope};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const CONSENT_LOG: &str = "reports/consent_log.json";
pub const FIGURE_GALLERY_OPERATION: &str = "figure_gallery_render";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentManifest {
    pub consent_id: Uuid,
    pub operation: String,
    pub data_categories: Vec<String>,
    pub endpoint: Option<String>,
    pub approved_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub approval_text: String,
    pub prompt_manifest: Value,
    pub orchestration_manifest_id: Option<Uuid>,
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
        self.persist_all(&manifests)
    }

    pub fn load_all(&self) -> Result<Vec<ConsentManifest>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn active_consent(&self, operation: &str) -> Result<Option<ConsentManifest>> {
        let now = Utc::now();
        Ok(self
            .load_all()?
            .into_iter()
            .rev()
            .find(|manifest| manifest.operation == operation && manifest.expires_at > now))
    }

    pub fn require_local_consent(
        &self,
        operation: &str,
        data_categories: &[&str],
        ttl_days: u32,
        approval_text: Option<&str>,
        prompt_manifest: Value,
    ) -> Result<ConsentManifest> {
        if let Some(active) = self.active_consent(operation)? {
            return Ok(active);
        }
        let text = approval_text.context(
            "Consent approval text is required when enabling this feature. Provide `--consent \"<reason>\"`.",
        )?;
        let manifest = ConsentManifest {
            consent_id: Uuid::new_v4(),
            operation: operation.to_string(),
            data_categories: data_categories.iter().map(|s| (*s).to_string()).collect(),
            endpoint: None,
            approved_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(ttl_days as i64),
            approval_text: text.to_string(),
            prompt_manifest,
            orchestration_manifest_id: None,
        };
        self.record(manifest.clone())?;
        Ok(manifest)
    }

    pub fn require_remote_consent(
        &self,
        consent_op: ConsentOperation,
        scope: ConsentScope,
        approval_text: &str,
        prompt_manifest: Value,
        ttl_days: u32,
        operation_alias: &str,
        data_categories: &[&str],
        endpoint: &str,
    ) -> Result<ConsentManifest> {
        let orchestration_manifest = require_remote_operation_consent(
            self.manager,
            &self.base,
            consent_op,
            approval_text,
            scope,
            prompt_manifest.clone(),
        )?;
        let manifest = ConsentManifest {
            consent_id: orchestration_manifest.manifest_id,
            operation: operation_alias.to_string(),
            data_categories: data_categories.iter().map(|s| (*s).to_string()).collect(),
            endpoint: Some(endpoint.to_string()),
            approved_at: orchestration_manifest.approved_at,
            expires_at: orchestration_manifest.approved_at + Duration::days(ttl_days as i64),
            approval_text: approval_text.to_string(),
            prompt_manifest,
            orchestration_manifest_id: Some(orchestration_manifest.manifest_id),
        };
        self.record(manifest.clone())?;
        Ok(manifest)
    }

    pub fn ensure_active(&self, operation: &str) -> Result<ConsentManifest> {
        self.active_consent(operation)?.with_context(|| {
            format!("Consent expired or missing for operation `{operation}`. Re-run `reports configure` to approve it again.")
        })
    }

    fn persist_all(&self, manifests: &[ConsentManifest]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let data = serde_json::to_string_pretty(manifests)?;
        fs::write(&self.path, data)?;
        Ok(())
    }
}
