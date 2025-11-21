use crate::bases::Base;
use crate::orchestration::intent::payload::{IntentPayload, IntentSafetyClass};
use serde_json::{json, Map, Number, Value};
use uuid::Uuid;

const INTENT_SCHEMA_VERSION: &str = "1.0.0";
const WRITING_INTENT_VERSION: &str = "0.1.0";

pub struct IntentParser;

impl IntentParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, base: &Base, chat_turn_id: Uuid, message: &str) -> Vec<IntentPayload> {
        let mut intents = Vec::new();
        for segment in split_segments(message) {
            if let Some(payload) = self.parse_segment(base, chat_turn_id, &segment) {
                intents.push(payload);
            }
        }
        intents
    }

    fn parse_segment(
        &self,
        base: &Base,
        chat_turn_id: Uuid,
        segment: &str,
    ) -> Option<IntentPayload> {
        let normalized = segment.trim();
        if normalized.is_empty() {
            return None;
        }
        if let Some(writing_intent) = self.parse_writing_command(base, chat_turn_id, normalized) {
            return Some(writing_intent);
        }
        let lower = normalized.to_ascii_lowercase();
        if lower.contains("summarize") && lower.contains("paper") {
            let count = extract_number(&lower).unwrap_or(3);
            return Some(build_summary_intent(base, chat_turn_id, normalized, count));
        }
        if lower.contains("delete") && lower.contains("profile") {
            let profile_type = detect_profile_type(&lower);
            return Some(build_profile_delete_intent(
                base,
                chat_turn_id,
                normalized,
                profile_type,
            ));
        }
        if lower.contains("show") && lower.contains("profile") {
            let profile_type = detect_profile_type(&lower);
            return Some(build_profile_show_intent(
                base,
                chat_turn_id,
                normalized,
                profile_type,
                lower.contains("history"),
            ));
        }
        if lower.contains("infer") && lower.contains("writing") {
            return Some(build_remote_infer_intent(base, chat_turn_id, normalized));
        }
        None
    }

    fn parse_writing_command(
        &self,
        base: &Base,
        chat_turn_id: Uuid,
        segment: &str,
    ) -> Option<IntentPayload> {
        if !segment.starts_with("/writing") {
            return None;
        }
        let after_prefix = segment["/writing".len()..].trim();
        if after_prefix.is_empty() {
            return Some(build_writing_help_intent(base, chat_turn_id, segment));
        }
        let normalized = after_prefix.replace('/', " ");
        let mut tokens = normalized.split_whitespace();
        let command = tokens.next()?.to_ascii_lowercase();
        match command.as_str() {
            "start" => {
                let title = after_prefix[command.len()..].trim();
                if title.is_empty() {
                    return None;
                }
                let mut parameters = Map::new();
                parameters.insert("title".into(), Value::String(strip_wrapping_tokens(title)));
                Some(build_writing_intent(
                    base,
                    chat_turn_id,
                    "writing.project.start",
                    segment,
                    parameters,
                    IntentSafetyClass::Destructive,
                    0.84,
                ))
            }
            "projects" => {
                if let Some(slug_token) = tokens.next() {
                    let slug = strip_identifier(slug_token);
                    let mut parameters = Map::new();
                    parameters.insert("project_slug".into(), Value::String(slug.clone()));
                    let next = tokens.next().map(|token| token.to_ascii_lowercase());
                    match next.as_deref() {
                        Some("style-interview") => {
                            return Some(build_writing_intent(
                                base,
                                chat_turn_id,
                                "writing.project.style_interview",
                                segment,
                                parameters,
                                IntentSafetyClass::Harmless,
                                0.78,
                            ));
                        }
                        Some("outline") => {
                            return Some(build_writing_intent(
                                base,
                                chat_turn_id,
                                "writing.project.outline",
                                segment,
                                parameters,
                                IntentSafetyClass::Harmless,
                                0.76,
                            ));
                        }
                        Some("drafts") => {
                            return Some(build_writing_intent(
                                base,
                                chat_turn_id,
                                "writing.project.drafts",
                                segment,
                                parameters,
                                IntentSafetyClass::Harmless,
                                0.74,
                            ));
                        }
                        Some("edit") => {
                            if let Some(section_id) = tokens.next() {
                                parameters.insert(
                                    "section_id".into(),
                                    Value::String(strip_identifier(section_id)),
                                );
                            }
                            return Some(build_writing_intent(
                                base,
                                chat_turn_id,
                                "writing.section.edit",
                                segment,
                                parameters,
                                IntentSafetyClass::Destructive,
                                0.72,
                            ));
                        }
                        Some("undo") => {
                            if let Some(event_id) = tokens.next() {
                                parameters.insert(
                                    "event_id".into(),
                                    Value::String(strip_identifier(event_id)),
                                );
                            }
                            return Some(build_writing_intent(
                                base,
                                chat_turn_id,
                                "writing.undo",
                                segment,
                                parameters,
                                IntentSafetyClass::Destructive,
                                0.68,
                            ));
                        }
                        Some("archive") => {
                            return Some(build_writing_intent(
                                base,
                                chat_turn_id,
                                "writing.project.archive",
                                segment,
                                parameters,
                                IntentSafetyClass::Destructive,
                                0.8,
                            ));
                        }
                        Some(other) => {
                            parameters.insert("operation".into(), Value::String(other.to_string()));
                            return Some(build_writing_intent(
                                base,
                                chat_turn_id,
                                "writing.project.update",
                                segment,
                                parameters,
                                IntentSafetyClass::Destructive,
                                0.7,
                            ));
                        }
                        None => {
                            return Some(build_writing_intent(
                                base,
                                chat_turn_id,
                                "writing.project.show",
                                segment,
                                parameters,
                                IntentSafetyClass::Harmless,
                                0.75,
                            ));
                        }
                    }
                } else {
                    Some(build_writing_intent(
                        base,
                        chat_turn_id,
                        "writing.projects.list",
                        segment,
                        Map::new(),
                        IntentSafetyClass::Harmless,
                        0.72,
                    ))
                }
            }
            "compile" => {
                let mut parameters = Map::new();
                if let Some(project) = tokens.next() {
                    parameters.insert(
                        "project_slug".into(),
                        Value::String(strip_identifier(project)),
                    );
                }
                Some(build_writing_intent(
                    base,
                    chat_turn_id,
                    "writing.compile",
                    segment,
                    parameters,
                    IntentSafetyClass::Harmless,
                    0.69,
                ))
            }
            "edit" => {
                let mut parameters = Map::new();
                if let Some(section_id) = tokens.next() {
                    parameters.insert(
                        "section_id".into(),
                        Value::String(strip_identifier(section_id)),
                    );
                }
                Some(build_writing_intent(
                    base,
                    chat_turn_id,
                    "writing.section.edit",
                    segment,
                    parameters,
                    IntentSafetyClass::Destructive,
                    0.7,
                ))
            }
            "undo" => {
                let mut parameters = Map::new();
                if let Some(event_id) = tokens.next() {
                    parameters.insert("event_id".into(), Value::String(strip_identifier(event_id)));
                }
                Some(build_writing_intent(
                    base,
                    chat_turn_id,
                    "writing.undo",
                    segment,
                    parameters,
                    IntentSafetyClass::Destructive,
                    0.68,
                ))
            }
            _ => Some(build_writing_help_intent(base, chat_turn_id, segment)),
        }
    }
}

fn split_segments(message: &str) -> Vec<String> {
    let normalized = message
        .replace(" then ", " and ")
        .replace(", then", " and")
        .replace("then,", "and");
    normalized
        .split(['.', ';'])
        .flat_map(|chunk| chunk.split(" and "))
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

fn extract_number(text: &str) -> Option<u32> {
    for token in text.split_whitespace() {
        if let Ok(value) = token.parse::<u32>() {
            return Some(value);
        }
    }
    None
}

fn detect_profile_type(segment: &str) -> Option<&'static str> {
    for keyword in ["writing", "knowledge", "work", "user"] {
        if segment.contains(keyword) {
            return Some(keyword);
        }
    }
    None
}

fn build_summary_intent(
    base: &Base,
    chat_turn_id: Uuid,
    segment: &str,
    count: u32,
) -> IntentPayload {
    let mut parameters = Map::new();
    parameters.insert("count".into(), Value::Number(Number::from(count)));
    IntentPayload::new(
        INTENT_SCHEMA_VERSION,
        "reports.generate_summary",
        json!({
            "scope": "recent_ingestions",
            "count": count,
            "source": segment
        }),
        parameters,
        0.92,
        IntentSafetyClass::Harmless,
        chat_turn_id,
        base.id,
    )
}

fn build_profile_show_intent(
    base: &Base,
    chat_turn_id: Uuid,
    segment: &str,
    profile_type: Option<&str>,
    include_history: bool,
) -> IntentPayload {
    let mut parameters = Map::new();
    if let Some(profile_type) = profile_type {
        parameters.insert(
            "profile_type".into(),
            Value::String(profile_type.to_string()),
        );
    }
    parameters.insert("include_history".into(), Value::Bool(include_history));
    let mut target = json!({ "source": segment });
    if let Some(profile_type) = profile_type {
        target["profile_type"] = Value::String(profile_type.to_string());
    }
    let confidence = if profile_type.is_some() { 0.88 } else { 0.62 };
    IntentPayload::new(
        INTENT_SCHEMA_VERSION,
        "profile.show",
        target,
        parameters,
        confidence,
        IntentSafetyClass::Harmless,
        chat_turn_id,
        base.id,
    )
}

fn build_profile_delete_intent(
    base: &Base,
    chat_turn_id: Uuid,
    segment: &str,
    profile_type: Option<&str>,
) -> IntentPayload {
    let mut parameters = Map::new();
    if let Some(profile_type) = profile_type {
        parameters.insert(
            "profile_type".into(),
            Value::String(profile_type.to_string()),
        );
    }
    let mut target = json!({ "source": segment });
    if let Some(profile_type) = profile_type {
        target["profile_type"] = Value::String(profile_type.to_string());
    }
    IntentPayload::new(
        INTENT_SCHEMA_VERSION,
        "profile.delete",
        target,
        parameters,
        0.91,
        IntentSafetyClass::Destructive,
        chat_turn_id,
        base.id,
    )
}

fn build_remote_infer_intent(base: &Base, chat_turn_id: Uuid, segment: &str) -> IntentPayload {
    let mut parameters = Map::new();
    parameters.insert("profile_type".into(), Value::String("writing".to_string()));
    IntentPayload::new(
        INTENT_SCHEMA_VERSION,
        "profile.remote_infer",
        json!({
            "profile_type": "writing",
            "manifest_summary": "Run remote inference to analyze writing tone.",
            "source": segment
        }),
        parameters,
        0.85,
        IntentSafetyClass::Remote,
        chat_turn_id,
        base.id,
    )
}

fn build_writing_intent(
    base: &Base,
    chat_turn_id: Uuid,
    action: &str,
    source: &str,
    parameters: Map<String, Value>,
    safety_class: IntentSafetyClass,
    confidence: f32,
) -> IntentPayload {
    let mut target = json!({ "source": source });
    if let Some(Value::String(slug)) = parameters.get("project_slug") {
        target["project_slug"] = Value::String(slug.clone());
    }
    IntentPayload::new(
        WRITING_INTENT_VERSION,
        action,
        target,
        parameters,
        confidence,
        safety_class,
        chat_turn_id,
        base.id,
    )
}

fn build_writing_help_intent(base: &Base, chat_turn_id: Uuid, source: &str) -> IntentPayload {
    IntentPayload::new(
        WRITING_INTENT_VERSION,
        "writing.help",
        json!({ "source": source }),
        Map::new(),
        0.4,
        IntentSafetyClass::Harmless,
        chat_turn_id,
        base.id,
    )
}

fn strip_wrapping_tokens(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string()
}

fn strip_identifier(token: &str) -> String {
    token
        .trim_matches(|c| c == '"' || c == '\'' || c == '{' || c == '}')
        .to_string()
}
