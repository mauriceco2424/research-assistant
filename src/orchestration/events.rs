use crate::bases::{Base, BaseManager};
use crate::orchestration::profiles::model::{ProfileChangeKind, ProfileType};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{EventType, OrchestrationEvent, OrchestrationLog};

/// Structured payload logged for every profile mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEventDetails {
    pub profile_type: ProfileType,
    pub change_kind: ProfileChangeKind,
    #[serde(default)]
    pub diff_summary: Vec<String>,
    pub hash_before: Option<String>,
    pub hash_after: Option<String>,
    pub undo_token: Option<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl ProfileEventDetails {
    pub fn new(
        profile_type: ProfileType,
        change_kind: ProfileChangeKind,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            profile_type,
            change_kind,
            diff_summary: Vec::new(),
            hash_before: None,
            hash_after: None,
            undo_token: None,
            payload,
        }
    }
}

pub fn log_profile_event(
    manager: &BaseManager,
    base: &Base,
    details: ProfileEventDetails,
) -> Result<Uuid> {
    let event_id = Uuid::new_v4();
    log_profile_event_with_id(manager, base, event_id, details)?;
    Ok(event_id)
}

pub fn log_profile_event_with_id(
    _manager: &BaseManager,
    base: &Base,
    event_id: Uuid,
    details: ProfileEventDetails,
) -> Result<Uuid> {
    let event = OrchestrationEvent {
        event_id,
        base_id: base.id,
        event_type: EventType::ProfileChange,
        timestamp: Utc::now(),
        details: serde_json::to_value(details)?,
    };
    let log = OrchestrationLog::for_base(base);
    log.append_event(&event)?;
    Ok(event.event_id)
}
