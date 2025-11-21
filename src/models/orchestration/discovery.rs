use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::discovery::{AcquisitionMode, AcquisitionOutcomeStatus, DiscoveryMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryEventDetails {
    pub request_id: Option<Uuid>,
    pub batch_id: Option<Uuid>,
    #[serde(default)]
    pub candidate_id: Option<Uuid>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub mode: Option<DiscoveryMode>,
    #[serde(default)]
    pub acquisition_mode: Option<AcquisitionMode>,
    #[serde(default)]
    pub prompt_manifest_path: Option<String>,
    #[serde(default)]
    pub endpoints_contacted: Vec<String>,
    #[serde(default)]
    pub outcome: Option<AcquisitionOutcomeStatus>,
    #[serde(default)]
    pub error_reason: Option<String>,
    #[serde(default)]
    pub timestamp: DateTime<Utc>,
}

impl DiscoveryEventDetails {
    pub fn new() -> Self {
        Self {
            request_id: None,
            batch_id: None,
            candidate_id: None,
            scope: None,
            mode: None,
            acquisition_mode: None,
            prompt_manifest_path: None,
            endpoints_contacted: Vec::new(),
            outcome: None,
            error_reason: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_request(
        mut self,
        request_id: Uuid,
        mode: DiscoveryMode,
        scope: impl Into<String>,
    ) -> Self {
        self.request_id = Some(request_id);
        self.mode = Some(mode);
        self.scope = Some(scope.into());
        self
    }

    pub fn with_batch(mut self, batch_id: Uuid, mode: AcquisitionMode) -> Self {
        self.batch_id = Some(batch_id);
        self.acquisition_mode = Some(mode);
        self
    }

    pub fn with_candidate(mut self, candidate_id: Uuid) -> Self {
        self.candidate_id = Some(candidate_id);
        self
    }

    pub fn with_manifest_path(mut self, path: impl Into<String>) -> Self {
        self.prompt_manifest_path = Some(path.into());
        self
    }

    pub fn with_endpoints(mut self, endpoints: Vec<String>) -> Self {
        self.endpoints_contacted = endpoints;
        self
    }

    pub fn with_outcome(
        mut self,
        outcome: AcquisitionOutcomeStatus,
        reason: Option<String>,
    ) -> Self {
        self.outcome = Some(outcome);
        self.error_reason = reason;
        self
    }
}
