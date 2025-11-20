pub fn no_match_response(message: &str) -> String {
    format!(
        "I couldn't route `{}` safely. Try explicit commands like `profile show writing` or `reports regenerate --scope recent`.",
        message.trim()
    )
}

pub fn manual_hint() -> String {
    "Need help? Run `help commands` or use `profile show writing` / `reports regenerate` directly."
        .into()
}

pub fn cancellation_message(failed_action: &str, pending: usize) -> String {
    format!("Cancelled {pending} queued intent(s) after `{failed_action}` failed.")
}
