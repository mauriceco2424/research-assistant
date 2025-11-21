use anyhow::{bail, Result};
use serde_json::json;
use uuid::Uuid;

use crate::{
    bases::{Base, BaseManager},
    chat::learning_sessions::{LearningSessionContext, LearningSessionSummary},
    orchestration::events::log_learning_undo_applied,
    profiles::LearningProfileUpdateResult,
    storage::ai_layer::LearningSessionStore,
};

pub fn summarize_session(
    base: &Base,
    session_id: Uuid,
    questions: Vec<serde_json::Value>,
    evaluations: Vec<serde_json::Value>,
    kp_deltas: Vec<String>,
    recommendations: Vec<String>,
) -> Result<LearningSessionSummary> {
    let summary = LearningSessionSummary {
        session_id,
        questions: Vec::new(),     // keep lightweight in summary; raw stored per question
        evaluations: Vec::new(),   // keep lightweight in summary; raw stored per question
        knowledge_profile_changes: kp_deltas,
        recommendations,
    };
    let store = LearningSessionStore::new(base);
    // Persist a lightweight summary plus raw artifacts separately if provided.
    store.save_summary(&session_id, &summary)?;
    if !questions.is_empty() {
        for (idx, q) in questions.iter().enumerate() {
            let path = store
                .session_root(&session_id)
                .join("questions_export")
                .join(format!("{idx}.json"));
            crate::storage::ai_layer::write_json(&path, q)?;
        }
    }
    if !evaluations.is_empty() {
        for (idx, e) in evaluations.iter().enumerate() {
            let path = store
                .session_root(&session_id)
                .join("evaluations_export")
                .join(format!("{idx}.json"));
            crate::storage::ai_layer::write_json(&path, e)?;
        }
    }
    Ok(summary)
}

pub fn render_summary_message(summary: &LearningSessionSummary) -> String {
    format!(
        "Session {} summary: {} KP updates, {} recommendations.",
        summary.session_id,
        summary.knowledge_profile_changes.len(),
        summary.recommendations.len()
    )
}

pub fn undo_latest_kp_update(
    manager: &BaseManager,
    base: &Base,
    context: &LearningSessionContext,
    last_update: Option<&LearningProfileUpdateResult>,
) -> Result<String> {
    let Some(update) = last_update else {
        bail!("No KnowledgeProfile updates available to undo.");
    };
    log_learning_undo_applied(
        manager,
        base,
        context.session_id,
        format!("{:?}", context.scope),
        format!("{:?}", context.mode),
        json!({ "undone_event": update.update.event_id }),
    )?;
    Ok(format!(
        "Undo applied for KnowledgeProfile update {} (session {}).",
        update.update.event_id, context.session_id
    ))
}
