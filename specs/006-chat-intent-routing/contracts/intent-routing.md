# Contracts – Chat Intent Routing APIs

## Overview
The assistant operates within the existing chat session, so most integrations are internal function calls rather than networked APIs. This contract documents the structured interactions between chat UI and orchestrator, plus capability registration hooks.

## 1. Intent Detection Pipeline

### Input (from chat UI to orchestrator)
```json
{
  "chat_turn_id": "uuid",
  "message": "Summarize the last 3 papers and show my writing profile",
  "base_id": "uuid"
}
```

### Response
```json
{
  "intents": [
    {
      "intent_id": "uuid",
      "action": "reports.generate_summary",
      "target": {
        "scope": "recent_ingestions",
        "count": 3
      },
      "parameters": {},
      "confidence": 0.92,
      "safety_class": "harmless",
      "status": "queued"
    },
    {
      "intent_id": "uuid",
      "action": "profile.show",
      "target": { "profile_type": "writing" },
      "parameters": { "include_history": true },
      "confidence": 0.88,
      "safety_class": "harmless",
      "status": "pending_previous"
    }
  ],
  "messages": [
    "Queued summary of last 3 papers; I'll report back with links.",
    "Will show writing profile after the summary finishes."
  ]
}
```

## 2. Confirmation Requests

### Request Payload
```json
{
  "intent_id": "uuid",
  "ticket": {
    "ticket_id": "uuid",
    "prompt": "Delete writing profile in Base demo-base?",
    "confirm_phrase": "DELETE writing",
    "safety_class": "destructive",
    "expires_at": "2025-11-20T15:30:00Z",
    "consent_manifest_ids": []
  }
}
```

### User Response
```json
{
  "ticket_id": "uuid",
  "decision": "approved",
  "phrase_entered": "DELETE writing"
}
```

Router updates ticket status and logs `intent_confirmed`.

## 3. Capability Registration Hook

Modules call:
```rust
register_capability(CapabilityDescriptor {
    descriptor_id: "profiles.scope",
    actions: vec!["profile.scope"],
    keywords: vec!["scope", "share profile"],
    required_params: vec!["profile_type"],
    validation_callback: profile_scope_validator,
    default_confirmation: ConfirmationRule::ConfirmPhrase("CONFIRM scope"),
    version: "1.0.0",
});
```

Registry guarantees:
- Duplicate `descriptor_id` rejected.
- Version recorded in intent events for replay.

## 4. Intent Event Log Schema

```json
{
  "event_id": "uuid",
  "event_type": "intent_executed",
  "intent_id": "uuid",
  "base_id": "uuid",
  "chat_turn_id": "uuid",
  "timestamp": "2025-11-20T15:32:05Z",
  "details": {
    "action": "profile.show",
    "result_event_id": "uuid",
    "undo_token": "undo://profile/show/..."
  }
}
```

## 5. Error / Fallback Responses

When confidence <0.80 or required parameters missing:
```json
{
  "clarification": {
    "intent_id": "uuid",
    "question": "Which profile should I delete?",
    "options": ["user", "work", "writing", "knowledge"]
  },
  "message": "I need a profile type before deleting anything. Pick one of the options."
}
```

If routing fails entirely:
```json
{
  "message": "I couldn't map that request safely. You can run commands manually, e.g., `profile show writing`.",
  "logged_event": "intent_failed"
}
```
