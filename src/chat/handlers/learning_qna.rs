use anyhow::{bail, Result};
use serde_json::json;
use uuid::Uuid;

use crate::{
    bases::{Base, BaseManager},
    chat::learning_sessions::{
        LearningEvaluation, LearningEvaluationOutcome, LearningMode, LearningQuestion,
        LearningSessionContext, DEFAULT_QUESTION_COUNT,
    },
    orchestration::events::{
        log_learning_answer_evaluated, log_learning_knowledge_updated, log_learning_question_generated,
    },
    profiles::{apply_learning_profile_updates, learning_undo_marker, LearningProfileUpdate},
    storage::ai_layer::LearningSessionStore,
};

fn ensure_active(context: &LearningSessionContext) -> Result<()> {
    if context.status != crate::chat::learning_sessions::LearningStatus::Active {
        bail!("Learning session is not active.");
    }
    Ok(())
}

pub fn next_question(
    manager: &BaseManager,
    base: &Base,
    context: &LearningSessionContext,
    prompt: String,
    target_concepts: Vec<String>,
    target_papers: Vec<Uuid>,
    difficulty: Option<String>,
    selection_rationale: Option<String>,
    expected_answer_outline: Option<String>,
) -> Result<(LearningQuestion, String)> {
    ensure_active(context)?;
    let mut question = LearningQuestion::new(prompt);
    question.target_concepts = target_concepts;
    question.target_papers = target_papers;
    question.difficulty = difficulty;
    question.selection_rationale = selection_rationale;
    question.expected_answer_outline = expected_answer_outline;

    let store = LearningSessionStore::new(base);
    store.save_question(&context.session_id, &question.question_id, &question)?;

    log_learning_question_generated(
        manager,
        base,
        context.session_id,
        question.question_id,
        format!("{:?}", context.scope),
        format!("{:?}", context.mode),
        json!({"selection_rationale": question.selection_rationale}),
    )?;

    let msg = format!(
        "Q{}: {}",
        question.question_id.to_string(),
        question.prompt
    );
    Ok((question, msg))
}

pub fn evaluate_answer(
    manager: &BaseManager,
    base: &Base,
    context: &LearningSessionContext,
    question: &LearningQuestion,
    user_answer: Option<String>,
    follow_ups: Vec<String>,
    kp_updates: Vec<LearningProfileUpdate>,
) -> Result<(LearningEvaluation, Option<String>, String)> {
    ensure_active(context)?;
    let outcome = if user_answer.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        LearningEvaluationOutcome::Partial
    } else {
        LearningEvaluationOutcome::Correct
    };

    let mut evaluation = LearningEvaluation::new(question.question_id, outcome);
    evaluation.user_answer = user_answer;
    evaluation.follow_up_recommendations = follow_ups;

    // Apply KnowledgeProfile updates if provided.
    let mut kp_update_ref: Option<String> = None;
    if !kp_updates.is_empty() {
        let result = apply_learning_profile_updates(manager, base, &kp_updates)?;
        kp_update_ref = Some(result.update.event_id.to_string());
        let undo_marker = learning_undo_marker(&result);
        evaluation.kp_update_ref = kp_update_ref.clone();

        log_learning_knowledge_updated(
            manager,
            base,
            context.session_id,
            Some(question.question_id),
            format!("{:?}", context.scope),
            format!("{:?}", context.mode),
            json!({ "update_event": result.update.event_id }),
        )?;

        // Link undo marker in evaluation payload for transparency.
        evaluation.feedback = Some(format!("Undo token: {}", undo_marker));
    }

    let store = LearningSessionStore::new(base);
    store.save_evaluation(&context.session_id, &question.question_id, &evaluation)?;

    log_learning_answer_evaluated(
        manager,
        base,
        context.session_id,
        question.question_id,
        format!("{:?}", context.scope),
        format!("{:?}", context.mode),
        json!({"outcome": format!("{:?}", evaluation.outcome)}),
    )?;

    let feedback = evaluation
        .feedback
        .clone()
        .unwrap_or_else(|| "Feedback recorded.".to_string());
    let msg = format!(
        "Evaluated question {} (outcome: {:?}). {}",
        question.question_id, evaluation.outcome, feedback
    );
    Ok((evaluation, kp_update_ref, msg))
}

pub fn enforce_default_question_window(
    asked_so_far: usize,
    default_question_count: usize,
) -> Option<&'static str> {
    if asked_so_far >= default_question_count {
        Some("Reached default question set. Continue with more questions or stop?")
    } else {
        None
    }
}

pub fn continue_or_stop_message(
    context: &LearningSessionContext,
    current_count: usize,
) -> String {
    let default_cap = context.default_question_count.max(DEFAULT_QUESTION_COUNT);
    if current_count >= default_cap {
        format!(
            "Default set ({default_cap}) complete. Say 'continue' for more or 'stop' to end."
        )
    } else {
        format!(
            "Answered {current_count}/{default_cap}. Continue to next question?"
        )
    }
}
