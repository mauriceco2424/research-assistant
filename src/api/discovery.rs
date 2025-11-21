use crate::bases::{Base, BaseManager};
use crate::models::discovery::{
    AcquisitionMode, DiscoveryApprovalBatch, DiscoveryCandidate, DiscoveryMode,
};
use crate::services::acquisition::discovery::approve_and_acquire;
use crate::services::discovery::gap::generate_gap_candidates;
use crate::services::discovery::session::generate_session_followups;
use crate::services::discovery::store::{load_request, persist_approval, persist_request};
use crate::services::discovery::topic::generate_topic_candidates;
use anyhow::{bail, Result};
use uuid::Uuid;

#[derive(Debug)]
pub struct DiscoveryRequestPayload {
    pub mode: DiscoveryMode,
    pub topic: Option<String>,
    pub gap_id: Option<String>,
    pub session_id: Option<Uuid>,
    pub count: Option<usize>,
}

#[derive(Debug)]
pub struct DiscoveryRequestResponse {
    pub request_id: Uuid,
    pub candidates: Vec<DiscoveryCandidate>,
}

#[derive(Debug)]
pub struct ApprovalPayload {
    pub request_id: Uuid,
    pub acquisition_mode: AcquisitionMode,
    pub candidate_ids: Vec<Uuid>,
    pub consent_manifest_path: Option<String>,
}

#[derive(Debug)]
pub struct ApprovalResponse {
    pub batch: DiscoveryApprovalBatch,
}

pub fn create_discovery_request(
    manager: &BaseManager,
    base: &Base,
    payload: DiscoveryRequestPayload,
) -> Result<DiscoveryRequestResponse> {
    let count = payload.count.unwrap_or(10);
    let candidates = match payload.mode {
        DiscoveryMode::Topic => {
            let topic = payload
                .topic
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("topic is required for topic mode"))?;
            generate_topic_candidates(manager, base, topic, count)?
        }
        DiscoveryMode::Gap => {
            let gap = payload
                .gap_id
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("gap_id is required for gap mode"))?;
            generate_gap_candidates(base.id, gap)?
        }
        DiscoveryMode::Session => {
            let session_id = payload
                .session_id
                .ok_or_else(|| anyhow::anyhow!("session_id is required for session mode"))?;
            generate_session_followups(session_id)?
        }
    };

    let record = persist_request(
        base,
        payload.mode,
        payload.topic,
        payload.gap_id,
        payload.session_id,
        candidates.clone(),
    )?;
    Ok(DiscoveryRequestResponse {
        request_id: record.request_id,
        candidates,
    })
}

pub fn approve_candidates(
    manager: &BaseManager,
    base: &Base,
    payload: ApprovalPayload,
) -> Result<ApprovalResponse> {
    if payload.candidate_ids.is_empty() {
        bail!("No candidates selected for approval");
    }
    let request = load_request(base, &payload.request_id)?
        .ok_or_else(|| anyhow::anyhow!("Discovery request {} not found", payload.request_id))?;
    let approval = persist_approval(
        base,
        payload.request_id,
        payload.candidate_ids.clone(),
        payload.acquisition_mode.clone(),
        payload.consent_manifest_path.clone(),
    )?;
    // Execute acquisition path and log outcomes.
    approve_and_acquire(manager, base, &request, &approval)?;
    Ok(ApprovalResponse { batch: approval })
}
