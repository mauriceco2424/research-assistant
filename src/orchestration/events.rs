use crate::bases::{Base, BaseManager};
use crate::models::orchestration::discovery::DiscoveryEventDetails;
use crate::orchestration::intent::payload::IntentPayload;
use crate::orchestration::profiles::model::{ProfileChangeKind, ProfileType};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    EventType, MetricRecord, OrchestrationEvent, OrchestrationLog, WritingMetricKind,
    WritingMetricRecord,
};

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

/// Structured payload logged for chat intent lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentEventDetails {
    pub intent_id: Uuid,
    pub action: String,
    pub chat_turn_id: Uuid,
    #[serde(default)]
    pub payload: serde_json::Value,
    #[serde(default)]
    pub confirmation_ticket_id: Option<Uuid>,
    #[serde(default)]
    pub result_event_id: Option<Uuid>,
    #[serde(default)]
    pub failure_reason: Option<String>,
}

impl IntentEventDetails {
    fn from_payload(payload: &IntentPayload) -> serde_json::Result<Self> {
        Ok(Self {
            intent_id: payload.intent_id,
            action: payload.action.clone(),
            chat_turn_id: payload.chat_turn_id,
            payload: serde_json::to_value(payload)?,
            confirmation_ticket_id: None,
            result_event_id: None,
            failure_reason: None,
        })
    }
}

pub fn log_intent_detected(base: &Base, payload: &IntentPayload) -> Result<Uuid> {
    let details = IntentEventDetails::from_payload(payload)?;
    log_intent_event(base, EventType::IntentDetected, details)
}

pub fn log_intent_confirmed(
    base: &Base,
    payload: &IntentPayload,
    confirmation_ticket_id: Uuid,
) -> Result<Uuid> {
    let mut details = IntentEventDetails::from_payload(payload)?;
    details.confirmation_ticket_id = Some(confirmation_ticket_id);
    log_intent_event(base, EventType::IntentConfirmed, details)
}

pub fn log_intent_executed(
    base: &Base,
    payload: &IntentPayload,
    result_event_id: Option<Uuid>,
) -> Result<Uuid> {
    let mut details = IntentEventDetails::from_payload(payload)?;
    details.result_event_id = result_event_id;
    log_intent_event(base, EventType::IntentExecuted, details)
}

pub fn log_intent_failed(
    base: &Base,
    payload: &IntentPayload,
    reason: impl Into<String>,
) -> Result<Uuid> {
    let mut details = IntentEventDetails::from_payload(payload)?;
    details.failure_reason = Some(reason.into());
    log_intent_event(base, EventType::IntentFailed, details)
}

fn log_intent_event(
    base: &Base,
    event_type: EventType,
    details: IntentEventDetails,
) -> Result<Uuid> {
    let event = OrchestrationEvent {
        event_id: Uuid::new_v4(),
        base_id: base.id,
        event_type,
        timestamp: Utc::now(),
        details: serde_json::to_value(details)?,
    };
    let log = OrchestrationLog::for_base(base);
    log.append_event(&event)?;
    Ok(event.event_id)
}

/// Structured payload for learning session lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEventDetails {
    pub session_id: Uuid,
    #[serde(default)]
    pub question_id: Option<Uuid>,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub default_question_count: Option<usize>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl LearningEventDetails {
    fn for_session(
        session_id: Uuid,
        scope: impl Into<String>,
        mode: impl Into<String>,
        default_question_count: usize,
    ) -> Self {
        Self {
            session_id,
            question_id: None,
            scope: scope.into(),
            mode: mode.into(),
            default_question_count: Some(default_question_count),
            payload: serde_json::Value::Null,
        }
    }

    fn for_question(
        session_id: Uuid,
        question_id: Uuid,
        scope: impl Into<String>,
        mode: impl Into<String>,
    ) -> Self {
        Self {
            session_id,
            question_id: Some(question_id),
            scope: scope.into(),
            mode: mode.into(),
            default_question_count: None,
            payload: serde_json::Value::Null,
        }
    }

    fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }
}

fn log_learning_event(
    base: &Base,
    event_type: EventType,
    details: LearningEventDetails,
) -> Result<Uuid> {
    let event = OrchestrationEvent {
        event_id: Uuid::new_v4(),
        base_id: base.id,
        event_type,
        timestamp: Utc::now(),
        details: serde_json::to_value(details)?,
    };
    let log = OrchestrationLog::for_base(base);
    log.append_event(&event)?;
    Ok(event.event_id)
}

pub fn log_learning_session_started(
    _manager: &BaseManager,
    base: &Base,
    session_id: Uuid,
    scope: impl Into<String>,
    mode: impl Into<String>,
    default_question_count: usize,
) -> Result<Uuid> {
    let details =
        LearningEventDetails::for_session(session_id, scope, mode, default_question_count);
    log_learning_event(base, EventType::LearningSessionStarted, details)
}

pub fn log_learning_question_generated(
    _manager: &BaseManager,
    base: &Base,
    session_id: Uuid,
    question_id: Uuid,
    scope: impl Into<String>,
    mode: impl Into<String>,
    rationale: serde_json::Value,
) -> Result<Uuid> {
    let details = LearningEventDetails::for_question(session_id, question_id, scope, mode)
        .with_payload(rationale);
    log_learning_event(base, EventType::LearningQuestionGenerated, details)
}

pub fn log_learning_answer_evaluated(
    _manager: &BaseManager,
    base: &Base,
    session_id: Uuid,
    question_id: Uuid,
    scope: impl Into<String>,
    mode: impl Into<String>,
    evaluation: serde_json::Value,
) -> Result<Uuid> {
    let details = LearningEventDetails::for_question(session_id, question_id, scope, mode)
        .with_payload(evaluation);
    log_learning_event(base, EventType::LearningAnswerEvaluated, details)
}

pub fn log_learning_knowledge_updated(
    _manager: &BaseManager,
    base: &Base,
    session_id: Uuid,
    question_id: Option<Uuid>,
    scope: impl Into<String>,
    mode: impl Into<String>,
    update_payload: serde_json::Value,
) -> Result<Uuid> {
    let mut details =
        LearningEventDetails::for_session(session_id, scope, mode, 0).with_payload(update_payload);
    details.question_id = question_id;
    details.default_question_count = None;
    log_learning_event(base, EventType::LearningKnowledgeUpdated, details)
}

pub fn log_learning_undo_applied(
    _manager: &BaseManager,
    base: &Base,
    session_id: Uuid,
    scope: impl Into<String>,
    mode: impl Into<String>,
    undo_payload: serde_json::Value,
) -> Result<Uuid> {
    let details =
        LearningEventDetails::for_session(session_id, scope, mode, 0).with_payload(undo_payload);
    log_learning_event(base, EventType::LearningUndoApplied, details)
}

/// ----------- Discovery orchestration logging -----------

fn log_discovery_event(
    base: &Base,
    event_type: EventType,
    details: DiscoveryEventDetails,
) -> Result<Uuid> {
    let event = OrchestrationEvent {
        event_id: Uuid::new_v4(),
        base_id: base.id,
        event_type,
        timestamp: Utc::now(),
        details: serde_json::to_value(details)?,
    };
    OrchestrationLog::for_base(base).append_event(&event)?;
    Ok(event.event_id)
}

pub fn log_discovery_request(
    _manager: &BaseManager,
    base: &Base,
    details: DiscoveryEventDetails,
) -> Result<Uuid> {
    log_discovery_event(base, EventType::DiscoveryRequestCreated, details)
}

pub fn log_discovery_approval(
    _manager: &BaseManager,
    base: &Base,
    request: &crate::models::discovery::DiscoveryRequestRecord,
    approval: &crate::models::discovery::DiscoveryApprovalBatch,
) -> Result<Uuid> {
    let details = DiscoveryEventDetails::new()
        .with_request(
            request.request_id,
            request.mode.clone(),
            request
                .topic
                .clone()
                .unwrap_or_else(|| "unspecified".to_string()),
        )
        .with_batch(approval.batch_id, approval.acquisition_mode.clone())
        .with_manifest_path(approval.consent_manifest_path.clone().unwrap_or_default());
    log_discovery_event(base, EventType::DiscoveryApprovalRecorded, details)
}

pub fn log_discovery_acquisition(
    _manager: &BaseManager,
    base: &Base,
    approval: &crate::models::discovery::DiscoveryApprovalBatch,
    _record: &crate::models::discovery::DiscoveryAcquisitionRecord,
) -> Result<Uuid> {
    let details = DiscoveryEventDetails::new()
        .with_batch(approval.batch_id, approval.acquisition_mode.clone())
        .with_manifest_path(
            approval
                .consent_manifest_path
                .clone()
                .unwrap_or_else(|| "".into()),
        );
    log_discovery_event(base, EventType::DiscoveryAcquisitionLogged, details)
}

/// Structured payload logged for writing assistant operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingEventDetails {
    pub project_slug: String,
    #[serde(default)]
    pub files_touched: Vec<String>,
    #[serde(default)]
    pub undo_checkpoint_path: Option<String>,
    #[serde(default)]
    pub consent_token: Option<String>,
    #[serde(default)]
    pub prompt_manifest_path: Option<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl WritingEventDetails {
    pub fn new(project_slug: impl Into<String>) -> Self {
        Self {
            project_slug: project_slug.into(),
            files_touched: Vec::new(),
            undo_checkpoint_path: None,
            consent_token: None,
            prompt_manifest_path: None,
            payload: serde_json::Value::Null,
        }
    }

    pub fn with_payload(project_slug: impl Into<String>, payload: serde_json::Value) -> Self {
        let mut details = Self::new(project_slug);
        details.payload = payload;
        details
    }

    pub fn with_files_touched<I, S>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.files_touched = files.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_undo_checkpoint(mut self, checkpoint: impl Into<String>) -> Self {
        self.undo_checkpoint_path = Some(checkpoint.into());
        self
    }

    pub fn with_consent_token(mut self, token: impl Into<String>) -> Self {
        self.consent_token = Some(token.into());
        self
    }

    pub fn with_prompt_manifest(mut self, path: impl Into<String>) -> Self {
        self.prompt_manifest_path = Some(path.into());
        self
    }
}

pub fn log_writing_project_created(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingProjectCreated, details)
}

pub fn log_style_model_ingested(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingStyleModelIngested, details)
}

pub fn log_outline_created(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingOutlineCreated, details)
}

pub fn log_outline_modified(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingOutlineModified, details)
}

pub fn log_draft_generated(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingDraftGenerated, details)
}

pub fn log_section_edited(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingSectionEdited, details)
}

pub fn log_citation_flagged(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingCitationFlagged, details)
}

pub fn log_compile_attempted(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingCompileAttempted, details)
}

pub fn log_writing_undo_applied(base: &Base, details: WritingEventDetails) -> Result<Uuid> {
    log_writing_event(base, EventType::WritingUndoApplied, details)
}

fn log_writing_event(
    base: &Base,
    event_type: EventType,
    details: WritingEventDetails,
) -> Result<Uuid> {
    let log = OrchestrationLog::for_base(base);
    maybe_record_writing_metric(&log, event_type.clone(), &details);
    let event = OrchestrationEvent {
        event_id: Uuid::new_v4(),
        base_id: base.id,
        event_type,
        timestamp: Utc::now(),
        details: serde_json::to_value(details)?,
    };
    log.append_event(&event)?;
    Ok(event.event_id)
}

fn maybe_record_writing_metric(
    log: &OrchestrationLog,
    event_type: EventType,
    details: &WritingEventDetails,
) {
    // Consent tracking
    if let Some(token) = &details.consent_token {
        let _ = log.record_metric(&MetricRecord::Writing(WritingMetricRecord {
            kind: WritingMetricKind::Consent,
            duration_ms: 0,
            success: true,
            details: serde_json::json!({
                "event": format!("{:?}", event_type),
                "consent_token": token,
                "prompt_manifest": details.prompt_manifest_path,
            }),
        }));
    }

    // Undo usage tracking
    if matches!(event_type, EventType::WritingUndoApplied) {
        let _ = log.record_metric(&MetricRecord::Writing(WritingMetricRecord {
            kind: WritingMetricKind::Undo,
            duration_ms: 0,
            success: true,
            details: serde_json::json!({
                "files": details.files_touched,
                "checkpoint": details.undo_checkpoint_path,
            }),
        }));
    }
}
