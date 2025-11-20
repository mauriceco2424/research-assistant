# Phase 0 Research – Chat Assistant & Intent Routing

## 1. Intent Payload Schema & Confidence Threshold
- **Decision**: Represent intents as JSON objects with `intent_version`, `action`, `target`, `parameters`, `confidence`, `safety_class`, `chat_turn_id`, and `base_id`. Use 0.80 confidence as the clarification threshold.
- **Rationale**: Fields mirror the functional requirements, enable deterministic replay, and 0.80 strikes a balance between accuracy and minimizing user interruptions.
- **Alternatives considered**:
  - *Minimal payload* (action + params only) – rejected because it would not support replay/undo or schema migrations.
  - *Higher threshold (0.90+)* – rejected due to excessive clarifications for long utterances.

## 2. Confirmation Workflow & Ticket Persistence
- **Decision**: Store confirmation prompts as `ConfirmationTicket` records containing the message shown to the user, confirm phrase, expiry timestamp, consent manifests (if any), and resulting decision. Tickets live next to intent logs per Base.
- **Rationale**: Provides auditable evidence for P2/P6, and allows the assistant to re-surface pending confirmations after reconnects.
- **Alternatives considered**:
  - *Ephemeral in-memory confirmations* – rejected because they would be lost on restart, violating regenerability.
  - *Global confirmation queue shared across Bases* – rejected to avoid cross-Base data leakage.

## 3. Capability Registration Strategy
- **Decision**: Implement a registry (`CapabilityDescriptor`) under `src/orchestration/intent/registry.rs` where each module (profiles, reports, learning, etc.) registers its intents (keywords, validation callback, confirmation rules).
- **Rationale**: Keeps router core slim, allows future specs to extend capabilities without editing the assistant, satisfying P10.
- **Alternatives considered**:
  - *Hard-coded match statements in router* – rejected for poor extensibility.
  - *External configuration files* – rejected to avoid user editing complexity and because most descriptors rely on Rust callbacks.

## 4. Contextual Suggestion Sources
- **Decision**: Build `SuggestionContext` snapshots by scanning AI-layer state the router already has access to (pending consent manifests, KnowledgeProfile entries flagged STALE, ingestion backlog metrics). Suggestions must cite the underlying evidence in responses.
- **Rationale**: Aligns with P7/P8 (integrity) and avoids speculative or hallucinated recommendations.
- **Alternatives considered**:
  - *LLM-generated suggestions without provenance* – rejected for violating transparency requirements.
  - *Manual user-configured suggestions* – rejected because it would shift burden away from the assistant.

## 5. Logging & Replay Integration
- **Decision**: Extend orchestration logging with `intent_detected`, `intent_confirmed`, `intent_executed`, `intent_failed` event types that reference underlying command events whenever available. Logs stored per Base in AI-layer JSONL.
- **Rationale**: Ensures replay/regeneration can reconstruct chat decisions and ties assistant actions to existing undo tokens.
- **Alternatives considered**:
  - *Only annotate existing command events* – rejected because missed intents (those that fail before dispatch) would never be recorded.
