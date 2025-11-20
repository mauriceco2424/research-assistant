use super::confirmation_flow::{ConfirmationKind, ConfirmationRequest};
use crate::orchestration::intent::payload::{IntentPayload, IntentSafetyClass};

const CLARIFICATION_THRESHOLD: f32 = 0.80;

pub struct SafetyClassifier;

pub enum SafetyDecision {
    Allow,
    RequireConfirmation(ConfirmationRequest),
    Clarification(ClarificationPrompt),
}

pub struct ClarificationPrompt {
    pub question: String,
    pub options: Vec<String>,
}

impl SafetyClassifier {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, payload: &IntentPayload) -> SafetyDecision {
        if payload.confidence < CLARIFICATION_THRESHOLD {
            return SafetyDecision::Clarification(
                self.build_clarification(payload, "confidence below 0.80"),
            );
        }

        if self.requires_profile_type(payload) {
            return SafetyDecision::Clarification(self.build_profile_prompt(payload));
        }

        match payload.safety_class {
            IntentSafetyClass::Destructive => {
                let target = profile_label(payload).unwrap_or_else(|| "profile".to_string());
                let prompt = format!("Delete the {target} profile in this Base?");
                let confirm_phrase = format!("DELETE {target}");
                SafetyDecision::RequireConfirmation(ConfirmationRequest {
                    prompt,
                    confirm_phrase,
                    kind: ConfirmationKind::Destructive {
                        target_label: target,
                    },
                    consent_manifest_ids: Vec::new(),
                })
            }
            IntentSafetyClass::Remote => {
                let manifest_summary = payload
                    .target
                    .get("manifest_summary")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string());
                let prompt = format!("Allow remote action `{}`?", payload.action);
                let confirm_phrase = format!("ALLOW {}", payload.action);
                SafetyDecision::RequireConfirmation(ConfirmationRequest {
                    prompt,
                    confirm_phrase,
                    kind: ConfirmationKind::Remote { manifest_summary },
                    consent_manifest_ids: Vec::new(),
                })
            }
            IntentSafetyClass::Harmless => SafetyDecision::Allow,
        }
    }

    fn requires_profile_type(&self, payload: &IntentPayload) -> bool {
        matches!(payload.action.as_str(), "profile.show" | "profile.delete")
            && profile_label(payload).is_none()
    }

    fn build_profile_prompt(&self, payload: &IntentPayload) -> ClarificationPrompt {
        ClarificationPrompt {
            question: format!(
                "Which profile should I target before running `{}`?",
                payload.action
            ),
            options: vec![
                "user".into(),
                "work".into(),
                "writing".into(),
                "knowledge".into(),
            ],
        }
    }

    fn build_clarification(&self, payload: &IntentPayload, reason: &str) -> ClarificationPrompt {
        ClarificationPrompt {
            question: format!(
                "I need more detail before running `{}` ({reason}).",
                payload.action
            ),
            options: Vec::new(),
        }
    }
}

fn profile_label(payload: &IntentPayload) -> Option<String> {
    payload
        .parameters
        .get("profile_type")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .or_else(|| {
            payload
                .target
                .get("profile_type")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
        })
}
