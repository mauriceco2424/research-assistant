# Intent Router Developer Guide

## Architecture
- **Parser (`src/chat/intent_router/parser.rs`)** splits chat turns into intent payloads (`IntentPayload`) with schema versioning, safety classes, and source metadata for replay.
- **Dispatcher (`src/chat/intent_router/dispatcher.rs`)** enforces the remote toggle, runs the safety classifier/confirmation flow, and logs every lifecycle event (`intent_detected`, `*_failed`, `suggestion.snapshot`) through `src/orchestration/events.rs`.
- **Persistence**:
  - Payloads append to `/AI/<Base>/intents/log.jsonl`.
  - Confirmation tickets live under `/AI/<Base>/intents/confirmations/`.
  - Suggestions reference AI-layer evidence paths so users can audit every recommendation.

## Capability Registration
1. Define a `CapabilityDescriptor` in `src/orchestration/intent/registry.rs`. Provide actions, keywords, required params, and optional validation callback.
2. Register during module initialization (`src/orchestration/mod.rs`) so the router can match segments without editing the core dispatcher.
3. When adding destructive or remote actions, set the `safety_class` in the emitted `IntentPayload`. The dispatcher will automatically route them through confirmations or remote guards.

### Example
```rust
use crate::orchestration::intent::registry::{CapabilityDescriptor, ConfirmationRule};

pub fn register_profile_show(registry: &mut CapabilityRegistry) {
    let mut descriptor = CapabilityDescriptor::new("profiles.show");
    descriptor.actions = vec!["profile.show".into()];
    descriptor.keywords = vec!["show profile".into(), "writing profile".into()];
    descriptor.required_params = vec!["profile_type".into()];
    descriptor.default_confirmation = ConfirmationRule::None;
    registry.register(descriptor).expect("duplicate descriptor");
}
```

## Suggestions & Fallback
- `SuggestionContext::build` aggregates pending consent manifests, stale knowledge entries, and ingestion backlog counts with evidence paths (`AI/<Base>/consent/manifests`, `AI/<Base>/profiles/knowledge.json`).
- `src/chat/intent_router/suggestions.rs` surfaces those signals when the user asks “What should I do next?”, citing file paths plus the `suggestion.snapshot` event id for replay.
- `src/chat/intent_router/fallback.rs` centralizes fallback messaging so unknown intents always guide users back to manual commands (`help commands`, `profile show writing`, etc.).

## Troubleshooting Checklist
| Symptom | Check |
|---------|-------|
| No intents detected | Confirm `/AI/<Base>/intents/log.jsonl` receives entries; parser may have ignored the utterance. |
| Confirmations never resolve | Inspect `/AI/<Base>/intents/confirmations/*.json` and ensure the confirm phrase matches the prompt exactly. |
| Suggestions missing evidence | Verify `knowledge.json` contains entries marked `verification_status = "stale"` or expired consent manifests exist under `consent/manifests/`. |
| Manual hint loops | User likely has no active Base; ensure `BaseManager::active_base` returns `Some` and quickstart step 1 is complete. |

## Related Files
- `src/chat/intent_router/*` – parsing, dispatcher, safety, suggestions, fallback UI.
- `src/orchestration/intent/*` – payload schema, logging, confirmation tickets, suggestion builder.
- `docs/perf/intent_router.md` – latest latency benchmarks and methodology.
