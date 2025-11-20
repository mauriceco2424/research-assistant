use crate::bases::Base;
use crate::chat::commands;
use crate::chat::intent_router::confirmation_flow::ConfirmationFlow;
use crate::chat::intent_router::fallback;
use crate::chat::intent_router::safety::{SafetyClassifier, SafetyDecision};
use crate::chat::intent_router::ui;
use crate::orchestration::events::{log_intent_detected, log_intent_executed, log_intent_failed};
use crate::orchestration::intent::payload::{IntentPayload, IntentSafetyClass};
use anyhow::{bail, Result};

pub trait IntentExecutor {
    fn execute_intent(&mut self, base: &Base, payload: &IntentPayload) -> Result<String>;
}

/// Dispatches intents to existing chat command handlers.
pub struct IntentDispatcher {
    remote_allowed: bool,
}

impl IntentDispatcher {
    pub fn new(remote_allowed: bool) -> Self {
        Self { remote_allowed }
    }

    pub fn run<E: IntentExecutor>(
        &self,
        executor: &mut E,
        base: &Base,
        intents: Vec<IntentPayload>,
    ) -> Result<Vec<String>> {
        if intents.is_empty() {
            return Ok(vec!["No actionable intents detected.".to_string()]);
        }

        let mut responses = Vec::new();
        let classifier = SafetyClassifier::new();
        let total = intents.len();
        for (index, intent) in intents.into_iter().enumerate() {
            log_intent_detected(base, &intent)?;
            if let Err(err) = self.guard_remote_policy(&intent) {
                let failure_id = log_intent_failed(base, &intent, err.to_string())?;
                responses.push(commands::format_intent_failure(
                    &intent.action,
                    &failure_id,
                    &err.to_string(),
                ));
                responses.push(fallback::manual_hint());
                if index < total - 1 {
                    responses.push(fallback::cancellation_message(
                        &intent.action,
                        total - index - 1,
                    ));
                }
                break;
            }

            match classifier.evaluate(&intent) {
                SafetyDecision::Clarification(prompt) => {
                    let failure_id = log_intent_failed(base, &intent, "clarification_required")?;
                    responses.push(ui::clarification_prompt(
                        &intent.action,
                        &prompt,
                        &failure_id,
                    ));
                    break;
                }
                SafetyDecision::RequireConfirmation(request) => {
                    let queued = ConfirmationFlow::queue(base, &intent, request)?;
                    let failure_id = log_intent_failed(base, &intent, "confirmation_required")?;
                    responses.push(ui::confirmation_prompt(
                        &intent.action,
                        &queued,
                        &failure_id,
                    ));
                    continue;
                }
                SafetyDecision::Allow => {}
            }

            match executor.execute_intent(base, &intent) {
                Ok(message) => {
                    let event_id = log_intent_executed(base, &intent, None)?;
                    responses.push(commands::format_intent_success(
                        &intent.action,
                        &event_id,
                        &message,
                    ));
                }
                Err(err) => {
                    let failure_id = log_intent_failed(base, &intent, err.to_string())?;
                    responses.push(commands::format_intent_failure(
                        &intent.action,
                        &failure_id,
                        &err.to_string(),
                    ));
                    responses.push(fallback::manual_hint());
                    if index < total - 1 {
                        responses.push(fallback::cancellation_message(
                            &intent.action,
                            total - index - 1,
                        ));
                    }
                    break;
                }
            }
        }

        Ok(responses)
    }

    /// Validates global remote access policy before executing the intent.
    pub fn guard_remote_policy(&self, payload: &IntentPayload) -> Result<()> {
        if payload.safety_class == IntentSafetyClass::Remote && !self.remote_allowed {
            bail!(
                "Remote intents are disabled for this installation. Enable remote access before running `{}`.",
                payload.action
            );
        }
        Ok(())
    }
}
