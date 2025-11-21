use crate::bases::Base;
use crate::models::discovery::{
    AcquisitionMode, DiscoveryApprovalBatch, DiscoveryMode, DiscoveryRequestRecord,
};
use crate::storage::ai_layer::DiscoveryStore;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

/// Persist a discovery request and return its id.
pub fn persist_request(
    base: &Base,
    mode: DiscoveryMode,
    topic: Option<String>,
    gap_id: Option<String>,
    session_id: Option<Uuid>,
    candidates: Vec<crate::models::discovery::DiscoveryCandidate>,
) -> Result<DiscoveryRequestRecord> {
    let store = DiscoveryStore::new(base);
    let record = DiscoveryRequestRecord {
        request_id: Uuid::new_v4(),
        base_id: base.id,
        mode,
        topic,
        gap_id,
        session_id,
        candidates,
        created_at: Utc::now(),
    };
    store.save_request(&record)?;
    Ok(record)
}

pub fn persist_approval(
    base: &Base,
    request_id: Uuid,
    candidate_ids: Vec<Uuid>,
    acquisition_mode: AcquisitionMode,
    consent_manifest_path: Option<String>,
) -> Result<DiscoveryApprovalBatch> {
    let store = DiscoveryStore::new(base);
    let batch = DiscoveryApprovalBatch {
        batch_id: Uuid::new_v4(),
        request_id,
        acquisition_mode,
        candidate_ids,
        consent_manifest_path,
        approved_at: Utc::now(),
    };
    store.save_approval(&batch)?;
    Ok(batch)
}

pub fn load_request(base: &Base, request_id: &Uuid) -> Result<Option<DiscoveryRequestRecord>> {
    let store = DiscoveryStore::new(base);
    store.load_request(request_id)
}
