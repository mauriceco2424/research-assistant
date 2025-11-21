use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Discovery request modes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMode {
    Topic,
    Gap,
    Session,
}

/// Acquisition modes users can choose at approval time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionMode {
    MetadataOnly,
    MetadataAndPdf,
}

/// Identifiers used for deduplication and provenance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryIdentifiers {
    #[serde(default)]
    pub doi: Option<String>,
    #[serde(default)]
    pub arxiv: Option<String>,
}

/// Duplicate match result when detecting existing papers in the Base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateMatch {
    pub matched_record_id: Uuid,
    #[serde(rename = "matched_via")]
    pub method: DuplicateMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateMethod {
    Doi,
    Arxiv,
    TitleAuthorYear,
}

/// Candidate paper proposed during discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryCandidate {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub source_link: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub identifiers: DiscoveryIdentifiers,
    #[serde(default)]
    pub duplicate_match: Option<DuplicateMatch>,
}

impl DiscoveryCandidate {
    pub fn metadata_summary(&self) -> String {
        format!(
            "{} ({})",
            self.title,
            self.year
                .map(|y| y.to_string())
                .unwrap_or_else(|| "n.d.".into())
        )
    }
}

/// Stored record for a discovery request with generated candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRequestRecord {
    pub request_id: Uuid,
    pub base_id: Uuid,
    pub mode: DiscoveryMode,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub gap_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<Uuid>,
    pub candidates: Vec<DiscoveryCandidate>,
    pub created_at: DateTime<Utc>,
}

/// Approval batch chosen by the user before acquisition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryApprovalBatch {
    pub batch_id: Uuid,
    pub request_id: Uuid,
    pub acquisition_mode: AcquisitionMode,
    pub candidate_ids: Vec<Uuid>,
    pub approved_at: DateTime<Utc>,
    #[serde(default)]
    pub consent_manifest_path: Option<String>,
}

/// Outcome per candidate after acquisition attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryAcquisitionOutcome {
    pub candidate_id: Uuid,
    pub outcome: AcquisitionOutcomeStatus,
    #[serde(default)]
    pub pdf_path: Option<String>,
    #[serde(default)]
    pub error_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionOutcomeStatus {
    Success,
    NeedsPdf,
    Skipped,
}

/// Persisted record for acquisition attempt associated with a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryAcquisitionRecord {
    pub batch_id: Uuid,
    pub base_id: Uuid,
    pub outcomes: Vec<DiscoveryAcquisitionOutcome>,
    pub recorded_at: DateTime<Utc>,
}
