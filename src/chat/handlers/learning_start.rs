use anyhow::{bail, Result};
use uuid::Uuid;

use crate::{
    bases::{Base, BaseManager},
    chat::learning_sessions::{
        LearningMode, LearningScope, LearningSessionContext, LearningStatus, DEFAULT_QUESTION_COUNT,
    },
    orchestration::events::log_learning_session_started,
    storage::ai_layer::LearningSessionStore,
};

fn scope_is_empty(scope: &LearningScope) -> bool {
    match scope {
        LearningScope::Base => false,
        LearningScope::Categories(list) => list.is_empty(),
        LearningScope::Papers(list) => list.is_empty(),
        LearningScope::Concepts(list) => list.is_empty(),
    }
}

fn scope_label(scope: &LearningScope) -> String {
    match scope {
        LearningScope::Base => "entire base".to_string(),
        LearningScope::Categories(list) => format!("categories: {}", list.join(", ")),
        LearningScope::Papers(list) => format!("papers: {}", list.len()),
        LearningScope::Concepts(list) => format!("concepts: {}", list.join(", ")),
    }
}

fn mode_label(mode: &LearningMode) -> &'static str {
    match mode {
        LearningMode::Quiz => "quiz",
        LearningMode::OralExam => "oral exam",
    }
}

/// Validates the requested learning scope and mode.
pub fn validate_scope_and_mode(scope: &LearningScope, mode: &LearningMode) -> Result<()> {
    if scope_is_empty(scope) {
        bail!("Selected scope is empty; pick a different scope or ingest more content.");
    }
    // Currently no special validation per mode beyond existence.
    match mode {
        LearningMode::Quiz | LearningMode::OralExam => Ok(()),
    }
}

/// Starts a learning session: validates, persists context, and logs orchestration.
pub fn start_learning_session(
    manager: &BaseManager,
    base: &Base,
    scope: LearningScope,
    mode: LearningMode,
    default_question_count: Option<usize>,
) -> Result<(LearningSessionContext, String)> {
    validate_scope_and_mode(&scope, &mode)?;
    let count = default_question_count.unwrap_or(DEFAULT_QUESTION_COUNT);
    if count == 0 {
        bail!("Default question count must be at least 1.");
    }
    let mut context = LearningSessionContext::new(base.id, scope.clone(), mode.clone())
        .with_default_question_count(count)
        .activate();

    // Persist context locally (AI layer).
    let store = LearningSessionStore::new(base);
    store.save_context(&context.session_id, &context)?;

    // Log orchestration event.
    log_learning_session_started(
        manager,
        base,
        context.session_id,
        scope_label(&scope),
        mode_label(&mode),
        count,
    )?;

    // Build chat confirmation.
    let msg = format!(
        "Started learning session {} (mode: {}, scope: {}, default questions: {}). All processing is local; approve any remote model use explicitly.",
        context.session_id,
        mode_label(&mode),
        scope_label(&scope),
        count,
    );

    context.status = LearningStatus::Active;
    Ok((context, msg))
}

/// Convenience helper for explicit session IDs in tests.
pub fn persist_learning_context(
    base: &Base,
    session_id: Uuid,
    context: &LearningSessionContext,
) -> Result<()> {
    let store = LearningSessionStore::new(base);
    store.save_context(&session_id, context)?;
    Ok(())
}
