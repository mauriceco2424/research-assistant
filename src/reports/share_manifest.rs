use crate::bases::{Base, BaseManager};
use crate::reports::manifest::ShareBundleDescriptor;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

const SHARE_MANIFEST_DIR: &str = "reports/share_manifests";

#[derive(Serialize)]
struct ShareManifestPayload<'a> {
    generated_at: String,
    descriptor: &'a ShareBundleDescriptor,
}

pub struct ShareManifestWriter<'a> {
    base: Base,
    _manager: &'a BaseManager,
}

impl<'a> ShareManifestWriter<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        Self {
            base: base.clone(),
            _manager: manager,
        }
    }

    pub fn persist(&self, descriptor: &ShareBundleDescriptor) -> Result<PathBuf> {
        let dir = self.base.ai_layer_path.join(SHARE_MANIFEST_DIR);
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
        let path = dir.join(format!("{}.json", descriptor.bundle_id));
        let payload = ShareManifestPayload {
            generated_at: Utc::now().to_rfc3339(),
            descriptor,
        };
        let data = serde_json::to_string_pretty(&payload)?;
        fs::write(&path, data)?;
        Ok(path)
    }
}
