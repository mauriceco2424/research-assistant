use crate::bases::Base;
use crate::orchestration::intent::{
    confirmation::{ConfirmationStore, ConfirmationTicket},
    payload::IntentPayload,
};
use anyhow::Result;
use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub enum ConfirmationKind {
    Destructive { target_label: String },
    Remote { manifest_summary: Option<String> },
}

pub struct ConfirmationRequest {
    pub prompt: String,
    pub confirm_phrase: String,
    pub kind: ConfirmationKind,
    pub consent_manifest_ids: Vec<Uuid>,
}

pub struct ConfirmationQueued {
    pub ticket: ConfirmationTicket,
    pub kind: ConfirmationKind,
}

pub struct ConfirmationFlow;

impl ConfirmationFlow {
    pub fn queue(
        base: &Base,
        payload: &IntentPayload,
        request: ConfirmationRequest,
    ) -> Result<ConfirmationQueued> {
        let store = ConfirmationStore::for_base(base)?;
        let ConfirmationRequest {
            prompt,
            confirm_phrase,
            kind,
            consent_manifest_ids,
        } = request;
        let expires_at = Utc::now() + Duration::minutes(15);
        let ticket = ConfirmationTicket::new(
            payload.intent_id,
            base.id,
            prompt,
            confirm_phrase,
            expires_at,
            consent_manifest_ids,
        );
        store.record(&ticket)?;
        Ok(ConfirmationQueued { ticket, kind })
    }
}
