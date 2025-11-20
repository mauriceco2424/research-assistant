use super::{
    confirmation_flow::{ConfirmationKind, ConfirmationQueued},
    safety::ClarificationPrompt,
};
use crate::chat::commands::{format_intent_failure, format_remote_consent_prompt};
use uuid::Uuid;

pub fn confirmation_prompt(action: &str, queued: &ConfirmationQueued, event_id: &Uuid) -> String {
    let base_message = match &queued.kind {
        ConfirmationKind::Destructive { target_label } => format!(
            "Destructive intent detected: delete the {target_label} profile. Reply with `{}` to approve.",
            queued.ticket.confirm_phrase
        ),
        ConfirmationKind::Remote { manifest_summary } => {
            let consent = format_remote_consent_prompt(manifest_summary.as_deref());
            format!(
                "{consent}\nReply with `{}` to approve remote access.",
                queued.ticket.confirm_phrase
            )
        }
    };
    format!(
        "{}\nTicket {} expires at {}. {}",
        base_message,
        queued.ticket.ticket_id,
        queued.ticket.expires_at.to_rfc3339(),
        format_intent_failure(action, event_id, "Awaiting confirmation.")
    )
}

pub fn clarification_prompt(action: &str, prompt: &ClarificationPrompt, event_id: &Uuid) -> String {
    let mut message = format!(
        "{}\n{}",
        prompt.question,
        format_intent_failure(action, event_id, "Clarification required.")
    );
    if !prompt.options.is_empty() {
        message.push_str("\nOptions: ");
        message.push_str(&prompt.options.join(", "));
    }
    message
}
