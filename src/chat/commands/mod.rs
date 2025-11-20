pub mod profile;
pub mod reports;

use uuid::Uuid;

pub fn format_intent_success(action: &str, event_id: &Uuid, details: &str) -> String {
    format!("[OK] {action} completed (intent event {event_id}). {details}")
}

pub fn format_intent_failure(action: &str, event_id: &Uuid, reason: &str) -> String {
    format!("[ERR] {action} failed (intent event {event_id}): {reason}")
}

pub fn format_remote_consent_prompt(summary: Option<&str>) -> String {
    let summary = summary.unwrap_or("This intent will send data to a remote AI service.");
    format!(
        "{summary} Consent reminder: review the manifest and confirm so the approval is logged locally."
    )
}
