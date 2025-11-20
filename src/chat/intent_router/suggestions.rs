use crate::bases::{Base, BaseManager};
use crate::chat::commands::format_intent_success;
use crate::orchestration::intent::suggestions::SuggestionContext;
use anyhow::Result;
use std::ffi::OsStr;

const SUGGESTION_TRIGGERS: &[&str] = &[
    "what should i do next",
    "what next",
    "suggest next",
    "recommend",
];

pub fn should_handle(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    SUGGESTION_TRIGGERS
        .iter()
        .any(|needle| lower.contains(needle))
}

pub fn build(manager: &BaseManager, base: &Base) -> Result<Vec<String>> {
    let context = SuggestionContext::build(manager, base)?;
    let mut responses = Vec::new();
    if !context.pending_consent.is_empty() {
        let consent_paths: Vec<String> = context
            .evidence_paths
            .iter()
            .filter(|path| {
                path.parent()
                    .and_then(|parent| parent.file_name())
                    .map(|name| name == OsStr::new("manifests"))
                    .unwrap_or(false)
            })
            .map(|path| path.display().to_string())
            .collect();
        responses.push(format!(
            "[Next] {count} consent manifest(s) need review. See {paths}.",
            count = context.pending_consent.len(),
            paths = if consent_paths.is_empty() {
                "AI/<Base>/consent/manifests".into()
            } else {
                consent_paths.join(", ")
            }
        ));
    }
    if !context.stale_knowledge_entries.is_empty() {
        let preview = context
            .stale_knowledge_entries
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let knowledge_path = context
            .evidence_paths
            .iter()
            .find(|path| {
                path.file_name()
                    .map(|name| name == OsStr::new("knowledge.json"))
                    .unwrap_or(false)
            })
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "AI/<Base>/profiles/knowledge.json".into());
        responses.push(format!(
            "[Next] Knowledge entries need refresh: {preview}. Source: {knowledge_path}."
        ));
    }
    if context.ingestion_backlog > 0 {
        responses.push(format!(
            "[Next] {count} papers still need PDFs. Run `ingest path-a` or attach manually.",
            count = context.ingestion_backlog
        ));
    }
    if responses.is_empty() {
        responses.push(
            "All systems look good. Ask for new papers or run `profile interview` to keep learning."
                .into(),
        );
    }
    responses.push(format_intent_success(
        "suggestion.snapshot",
        &context.snapshot_id,
        &format!(
            "Generated at {} with {} evidence reference(s).",
            context.generated_at.to_rfc3339(),
            context.evidence_paths.len()
        ),
    ));
    Ok(responses)
}
