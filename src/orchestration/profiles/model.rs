//! Data structures backing AI profile orchestration.
//!
//! These types mirror `specs/005-ai-profile-memory/data-model.md` so storage,
//! orchestration, and chat flows can share a consistent contract.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProfileType {
    User,
    Work,
    Writing,
    Knowledge,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileScopeMode {
    ThisBase,
    Shared,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub id: String,
    pub last_updated: DateTime<Utc>,
    pub scope: ProfileScopeMode,
    #[serde(default)]
    pub allowed_bases: Vec<String>,
}

impl ProfileMetadata {
    pub fn new(id: impl Into<String>, scope: ProfileScopeMode) -> Self {
        Self {
            id: id.into(),
            last_updated: Utc::now(),
            scope,
            allowed_bases: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRef {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub hash_after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileFields {
    pub name: String,
    #[serde(default)]
    pub affiliations: Vec<String>,
    #[serde(default)]
    pub communication_style: Vec<String>,
    pub availability: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub metadata: ProfileMetadata,
    #[serde(default)]
    pub summary: Vec<String>,
    pub fields: UserProfileFields,
    #[serde(default)]
    pub history: Vec<HistoryRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRef {
    pub name: String,
    pub status: Option<String>,
    pub target_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneRef {
    pub description: String,
    pub due: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProfileFields {
    #[serde(default)]
    pub active_projects: Vec<ProjectRef>,
    #[serde(default)]
    pub milestones: Vec<MilestoneRef>,
    #[serde(default)]
    pub preferred_tools: Vec<String>,
    pub focus_statement: Option<String>,
    #[serde(default)]
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProfile {
    pub metadata: ProfileMetadata,
    #[serde(default)]
    pub summary: Vec<String>,
    pub fields: WorkProfileFields,
    #[serde(default)]
    pub history: Vec<HistoryRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleExample {
    pub source: String,
    pub excerpt: String,
    pub citation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInferenceMetadata {
    pub last_remote_source: Option<String>,
    pub consent_manifest_id: Option<Uuid>,
    #[serde(default)]
    pub status: RemoteInferenceStatus,
}

impl Default for RemoteInferenceMetadata {
    fn default() -> Self {
        Self {
            last_remote_source: None,
            consent_manifest_id: None,
            status: RemoteInferenceStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemoteInferenceStatus {
    Approved,
    Rejected,
    Pending,
}

impl Default for RemoteInferenceStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingProfileFields {
    #[serde(default)]
    pub tone_descriptors: Vec<String>,
    #[serde(default)]
    pub structure_preferences: Vec<String>,
    #[serde(default)]
    pub style_examples: Vec<StyleExample>,
    #[serde(default)]
    pub remote_inference_metadata: RemoteInferenceMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingProfile {
    pub metadata: ProfileMetadata,
    #[serde(default)]
    pub summary: Vec<String>,
    pub fields: WritingProfileFields,
    #[serde(default)]
    pub history: Vec<HistoryRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    #[serde(rename = "type")]
    pub kind: EvidenceKind,
    pub identifier: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Paper,
    Note,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningLink {
    pub task_id: String,
    pub title: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MasteryLevel {
    Novice,
    Developing,
    Proficient,
    Expert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub concept: String,
    pub mastery_level: MasteryLevel,
    #[serde(default)]
    pub evidence_refs: Vec<EvidenceRef>,
    #[serde(default)]
    pub weakness_flags: Vec<String>,
    #[serde(default)]
    pub learning_links: Vec<LearningLink>,
    pub last_reviewed: DateTime<Utc>,
    pub verification_status: VerificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeProfile {
    pub metadata: ProfileMetadata,
    #[serde(default)]
    pub summary: Vec<String>,
    #[serde(default)]
    pub entries: Vec<KnowledgeEntry>,
    #[serde(default)]
    pub history: Vec<HistoryRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileChangeEvent {
    pub event_id: Uuid,
    pub profile_type: ProfileType,
    pub change_kind: ProfileChangeKind,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    #[serde(default)]
    pub diff_summary: Vec<String>,
    pub undo_token: Option<String>,
    #[serde(default)]
    pub consent_manifest_ids: Vec<Uuid>,
    pub hash_before: Option<String>,
    pub hash_after: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileChangeKind {
    Create,
    Interview,
    ManualEdit,
    ScopeChange,
    Export,
    Delete,
    Regenerate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentManifest {
    pub manifest_id: Uuid,
    pub operation_type: String,
    #[serde(default)]
    pub data_categories: Vec<String>,
    pub provider: String,
    pub prompt_excerpt: String,
    pub approved_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub profiles_touched: Vec<ProfileType>,
    pub status: ConsentStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    Approved,
    Rejected,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileScopeSetting {
    pub profile_type: ProfileType,
    pub scope_mode: ProfileScopeMode,
    #[serde(default)]
    pub allowed_bases: Vec<String>,
    pub updated_at: DateTime<Utc>,
}
