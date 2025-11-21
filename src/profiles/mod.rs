use anyhow::{bail, Result};
use uuid::Uuid;

use crate::{
    bases::{Base, BaseManager},
    orchestration::profiles::model::ProfileType,
    orchestration::profiles::service::{ProfileFieldChange, ProfileService, ProfileUpdateOutput},
};

pub mod writing_profile;

/// Session-scoped KnowledgeProfile update, tied to a learning session/question.
#[derive(Debug, Clone)]
pub struct LearningProfileUpdate {
    pub session_id: Uuid,
    pub question_id: Option<Uuid>,
    pub concept: String,
    pub field: String,
    pub value: String,
}

impl LearningProfileUpdate {
    pub fn into_field_change(self) -> ProfileFieldChange {
        ProfileFieldChange::new(format!("{}.{}", self.concept, self.field), self.value)
    }
}

/// Result of applying session-scoped KnowledgeProfile updates.
#[derive(Debug, Clone)]
pub struct LearningProfileUpdateResult {
    pub session_id: Uuid,
    pub question_id: Option<Uuid>,
    pub update: ProfileUpdateOutput,
}

/// Apply KnowledgeProfile updates triggered by a learning session/question.
pub fn apply_learning_profile_updates(
    manager: &BaseManager,
    base: &Base,
    updates: &[LearningProfileUpdate],
) -> Result<LearningProfileUpdateResult> {
    if updates.is_empty() {
        bail!("No learning profile updates provided.");
    }
    let session_id = updates[0].session_id;
    let question_id = updates[0].question_id;
    if updates
        .iter()
        .any(|u| u.session_id != session_id || u.question_id != question_id)
    {
        bail!("All learning updates in a single call must share the same session/question context.");
    }
    let changes: Vec<ProfileFieldChange> = updates
        .iter()
        .cloned()
        .map(LearningProfileUpdate::into_field_change)
        .collect();
    let service = ProfileService::new(manager, base);
    let update = service.update(ProfileType::Knowledge, &changes, true)?;
    Ok(LearningProfileUpdateResult {
        session_id,
        question_id,
        update,
    })
}

/// Returns an undo marker for the applied KnowledgeProfile change (uses event id).
pub fn learning_undo_marker(result: &LearningProfileUpdateResult) -> String {
    result.update.event_id.to_string()
}
