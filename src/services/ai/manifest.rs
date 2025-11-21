use crate::bases::{Base, BaseManager};
use crate::orchestration::consent::store::ConsentStore;
use crate::orchestration::ConsentManifest;
use anyhow::Result;
use serde_json::Value;
use uuid::Uuid;

/// Records a prompt manifest for discovery-related remote inference and logs the consent event.
pub fn record_discovery_manifest(
    _manager: &BaseManager,
    base: &Base,
    manifest: ConsentManifest,
) -> Result<Uuid> {
    let store = ConsentStore::for_base(base);
    store.record(&manifest)?;
    Ok(manifest.manifest_id)
}

/// Helper to build a manifest JSON blob for discovery based on provided metadata.
pub fn build_discovery_prompt_manifest(
    operation: &str,
    data_categories: &[&str],
    destination: &str,
) -> Value {
    serde_json::json!({
        "operation": operation,
        "data_categories": data_categories,
        "destination": destination,
    })
}
