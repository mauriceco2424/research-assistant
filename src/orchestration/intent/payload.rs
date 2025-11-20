//! Intent payload definitions for the chat assistant router.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

/// Serialized representation of a parsed intent emitted by the chat router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPayload {
    pub intent_id: Uuid,
    pub intent_version: String,
    pub action: String,
    #[serde(default = "default_target")]
    pub target: Value,
    #[serde(default = "default_parameters")]
    pub parameters: Map<String, Value>,
    pub confidence: f32,
    pub safety_class: IntentSafetyClass,
    pub chat_turn_id: Uuid,
    pub base_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl IntentPayload {
    /// Constructs a new payload populated with the provided metadata.
    pub fn new(
        intent_version: impl Into<String>,
        action: impl Into<String>,
        target: Value,
        parameters: Map<String, Value>,
        confidence: f32,
        safety_class: IntentSafetyClass,
        chat_turn_id: Uuid,
        base_id: Uuid,
    ) -> Self {
        Self {
            intent_id: Uuid::new_v4(),
            intent_version: intent_version.into(),
            action: action.into(),
            target,
            parameters,
            confidence: confidence.clamp(0.0, 1.0),
            safety_class,
            chat_turn_id,
            base_id,
            created_at: Utc::now(),
        }
    }

    /// Serializes the payload into a JSON line (no trailing newline).
    pub fn to_json_line(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Parses a payload from a previously serialized JSON line.
    pub fn from_json_line(line: &str) -> serde_json::Result<Self> {
        serde_json::from_str(line)
    }
}

fn default_target() -> Value {
    Value::Null
}

fn default_parameters() -> Map<String, Value> {
    Map::new()
}

/// Classification for router safety decisions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentSafetyClass {
    Harmless,
    Destructive,
    Remote,
}
