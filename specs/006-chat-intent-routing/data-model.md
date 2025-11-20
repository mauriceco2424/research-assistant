# Data Model – Chat Assistant & Intent Routing

## Entities

### IntentPayload
- **Fields**
  | Field | Type | Notes |
  |-------|------|-------|
  | `intent_version` | string | Semantic version of schema |
  | `action` | string | Canonical command identifier (`profile.show`, `reports.generate`) |
  | `target` | string or object | Target entity (profile type, report scope, paper id) |
  | `parameters` | map | Key-value args parsed from chat |
  | `confidence` | float (0–1) | Router confidence score |
  | `safety_class` | enum { `harmless`, `destructive`, `remote` } | Drives confirmation policy |
  | `chat_turn_id` | UUID | Links back to chat transcript |
  | `base_id` | UUID | Active Base |
  | `created_at` | ISO-8601 | Timestamp when intent parsed |
- **Relationships**: Linked to zero or more ConfirmationTickets and IntentEvent log entries.

### ConfirmationTicket
- **Fields**
  | Field | Type | Notes |
  |-------|------|-------|
  | `ticket_id` | UUID | Unique per confirmation |
  | `prompt` | string | Text shown to user |
  | `confirm_phrase` | string | Required phrase or button label |
  | `expires_at` | ISO-8601 | Ticket timeout |
  | `status` | enum { `pending`, `approved`, `denied`, `expired` } |
  | `consent_manifest_ids` | array<UUID> | Remote inference links |
  | `intent_id` | UUID | Foreign key to IntentPayload |
- **Relationships**: One-to-one with the intent requiring confirmation; stored per Base.

### CapabilityDescriptor
- **Fields**
  | Field | Type | Notes |
  |-------|------|-------|
  | `descriptor_id` | string | Module-provided identifier |
  | `actions` | array<string> | Canonical command names |
  | `keywords` | array<string> | Hints for NLP matching |
  | `required_params` | array<string> | e.g., profile type |
  | `validation_callback` | function ref | Invoked before dispatch |
  | `default_confirmation` | enum | `none`, `confirm_phrase`, `manifest` |
- **Relationships**: Registered once per module; referenced when router resolves intents.

### IntentEvent
- **Fields**
  | Field | Type | Notes |
  |-------|------|-------|
  | `event_id` | UUID | Shared with orchestration log |
  | `event_type` | enum { `intent_detected`, `intent_confirmed`, `intent_executed`, `intent_failed` } |
  | `intent_id` | UUID | Link to payload |
  | `timestamp` | ISO-8601 | Event time |
  | `details` | JSON | Includes undo tokens, failure reasons |
- **Relationships**: Stored alongside orchestration log for each Base.

### SuggestionContext
- **Fields**
  | Field | Type | Notes |
  |-------|------|-------|
  | `snapshot_id` | UUID | Generated per assistant cycle |
  | `pending_consent` | array<UUID> | Manifest ids awaiting review |
  | `stale_knowledge_entries` | array<string> | Concept ids flagged STALE |
  | `ingestion_backlog` | integer | Count of unprocessed papers |
  | `generated_at` | ISO-8601 | Timestamp |
- **Relationships**: Not persisted long-term; regenerated on demand from AI-layer data.

## State & Lifecycle
- **IntentPayload**: `created` → (`queued` | `aborted`). Successful intents move to `queued`, then `executed` after confirmation + dispatch. Aborted intents still logged via `intent_failed`.
- **ConfirmationTicket**: `pending` until user replies; transitions to `approved`, `denied`, or `expired`. Tickets are referenced in chat responses and stored for audit.
- **CapabilityDescriptor**: Registered at startup; may be hot-reloaded if module updates the registry. Version metadata ensures compatibility.
- **IntentEvent**: Append-only; replay uses chronological ordering to reconstruct assistant decisions.

## Validation Rules
- Confidence <0.80 requires clarification UI before dispatch.
- Destructive (`safety_class = destructive`) intents always require confirm phrase referencing target (e.g., “DELETE writing”).
- Remote intents must list consent manifest ids; router cannot proceed without recorded approval.
- Registry must reject duplicate descriptor ids to avoid ambiguity.
- SuggestionContext items must cite underlying AI-layer file paths to maintain transparency.
