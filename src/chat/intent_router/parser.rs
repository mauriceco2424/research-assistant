use crate::bases::Base;
use crate::orchestration::intent::payload::{IntentPayload, IntentSafetyClass};
use serde_json::{json, Map, Number, Value};
use uuid::Uuid;

const INTENT_SCHEMA_VERSION: &str = "1.0.0";

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
