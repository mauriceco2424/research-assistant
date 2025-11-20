# Feature Specification: Chat Assistant & Intent Routing

**Feature Branch**: `006-chat-intent-routing`  
**Created**: 2025-11-20  
**Status**: Draft  
**Input**: User description: "Design the Chat Assistant + Intent Routing capability so natural-language inputs are parsed, confirmed, and dispatched to existing commands while honoring P1-P10 and the dual-layer logging model."

## Clarifications

### Session 2025-11-20

- Q: What default intent-confidence threshold should trigger clarification? → A: 0.80 confidence threshold

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Natural chat → command execution (Priority: P1)

A researcher types a multi-part natural-language instruction (“Summarize the last 3 papers I imported and then show my writing profile”). The assistant must parse intents in order, confirm any long-running or destructive steps, and execute the mapped commands without requiring the user to remember CLI syntax.

**Why this priority**: This is the core value of Spec 06—making the chat panel the universal control surface so every other feature remains reachable from conversational input.

**Independent Test**: Start from a seeded Base, issue multi-intent chat requests, and verify the assistant produces confirmations, executes mapped commands (reports, profile show), and returns orchestration references without manual CLI invocations.

**Acceptance Scenarios**:

1. **Given** a Base with recent ingestion events, **When** the user says “Summarize the last 3 papers I imported,” **Then** the assistant confirms the scope, triggers the report command, and replies with links plus the orchestration event id.
2. **Given** an existing writing profile, **When** the user appends "then show my writing profile," **Then** the assistant queues the profile show command after the summary completes and reports both outcomes in chat with timestamps.
3. **Given** a multi-intent message, **When** the first intent fails or is denied, **Then** the assistant cancels the remaining intents, explains why they were skipped, and logs `intent_failed` for the aborted steps.

---

### User Story 2 - Safety gating & clarifications (Priority: P2)

When a user issues ambiguous or destructive commands (“delete the profile”, “purge yesterday’s papers”), the assistant must clarify the target, highlight the confirm phrase, surface constitutional safeguards (local-first, undo availability), and only execute once the user explicitly approves.

**Why this priority**: Prevents accidental data loss and enforces constitutional principles (P1, P6) for every routed action.

**Independent Test**: Attempt to delete or export resources using vague chat instructions and confirm the assistant blocks execution until the user selects an explicit option and confirm phrase; log entries must show the confirmation token.

**Acceptance Scenarios**:

1. **Given** multiple profiles exist, **When** a user says “delete the profile,” **Then** the assistant responds with a clarification card listing eligible profiles, required confirm phrases, and warns about local-only deletion before proceeding.
2. **Given** the user approves “DELETE writing,” **When** the assistant executes the command, **Then** the chat response includes the event id, undo instructions, and pointers to the audit log.

---

### User Story 3 - Context-aware suggestions & fallback (Priority: P3)

If the user expresses intent of “What should I do next?” or provides an unclear request, the assistant inspects AI-layer signals (pending consent manifests, stale knowledge entries, recent ingestions) to recommend actionable options and gracefully handles low-confidence interpretations.

**Why this priority**: Keeps the chat experience proactive and resilient even when the user is unsure or the router lacks confidence, reducing dead-ends.

**Independent Test**: Issue vague or open-ended prompts and observe that the assistant surfaces contextual suggestions (e.g., “Review pending consent,” “Run learning prep”) and explains how to continue without throwing errors.

**Acceptance Scenarios**:

1. **Given** there are pending consent manifests, **When** the user asks “Anything I should review?”, **Then** the assistant lists pending approvals with commands to proceed.
2. **Given** no confident intent match, **When** the user types an ambiguous request, **Then** the assistant presents clarifying questions and a fallback option to run the command manually, logging the failed intent attempt.

---

### Edge Cases

- If multiple intents conflict (e.g., “delete the profile and export it”), the assistant must warn about incompatible ordering and request a resolution before running any action.
- When the user is offline or no Base is selected, attempts to route commands must return a chat notice explaining the missing context and how to recover.
- If intent confidence falls below the configured threshold, the assistant must avoid guessing and instead offer clarification choices plus the option to type explicit command syntax.
- When remote inference is required, the assistant must summarize the prompt manifest, capture consent, and cache the approval token for the corresponding orchestration event.
- On undo requests, the assistant must verify a matching event id exists and surface next steps if undo is unavailable (e.g., aged-out operations).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The assistant MUST accept free-form chat input and produce a structured intent payload containing action, target, parameters, confidence, safety classification, and originating chat turn id.
- **FR-002**: The router MUST support multi-intent utterances by maintaining an ordered queue and executing each intent only after prerequisites and confirmations succeed.
- **FR-003**: For destructive or remote actions, the assistant MUST present human-readable confirmations (including Base, target, and implications) and require explicit approval before dispatch.
- **FR-004**: The system MUST provide clarification prompts when confidence is below the configured 0.80 threshold or when required parameters (Base, profile type, count) are missing.
- **FR-005**: Each routed intent MUST emit orchestration events (`intent_detected`, `intent_confirmed`, `intent_executed`, `intent_failed`) capturing Base id, chat turn id, consent manifest ids (if any), and undo references.
- **FR-006**: The assistant MUST expose fallback guidance (“Run `profile show writing`”) whenever it cannot safely route an intent, ensuring the user can proceed manually.
- **FR-007**: Capability providers (profiles, reports, ingestion, learning, future modules) MUST be able to register their command descriptors (name, intent keywords, validation rules, required confirmations) without modifying the assistant core.
- **FR-008**: The assistant MUST enforce that all prompts, consent manifests, and intent logs remain local unless the user explicitly approves remote inference, satisfying P1/P2, and MUST block any `safety_class = remote` intents whenever the global “remote AI disabled” toggle is active.
- **FR-009**: Contextual suggestions MUST be generated from AI-layer state (e.g., pending consent, stale knowledge entries, recent ingestion gaps) with clear explanations for why each suggestion appears.
- **FR-010**: The assistant MUST preserve execution order and stop subsequent intents when a preceding one fails or is declined, reporting the reason to the user.

### Key Entities *(include if feature involves data)*

- **IntentPayload**: Structured record representing a parsed user instruction; includes action name, parameters, source chat turn id, confidence score, safety class, and linked Base id.
- **ConfirmationTicket**: Captures the text shown to the user, the confirm phrase or quick-response choice, expiration timestamp, consent manifest ids, and the resulting decision (approved/denied).
- **CapabilityDescriptor**: Registration contract provided by each feature module describing intent keywords, required parameters, validation callbacks, and default confirmation rules.
- **IntentEvent Log Entry**: Append-only record stored in the AI-layer capturing `intent_detected/confirmed/executed/failed`, orchestration event ids, undo guidance, and any remote-manifest references.
- **SuggestionContext Snapshot**: Derived data summarizing pending actions (consent to review, stale knowledge entries, incomplete ingestions) that feed the assistant’s proactive recommendations.

### Assumptions & Dependencies

- The chat panel remains the sole user-facing interface; no additional UI panels are introduced.
- Existing commands (e.g., `profile show`) already produce orchestration events and undo instructions that the assistant can surface.
- The Base manager guarantees exactly one active Base per chat session; if none is selected the assistant can prompt the user to choose.
- Future specs (Writing Assistant, Learning Mode) will register their capabilities through the same descriptor format defined here.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 95% of routed intents with confidence ≥ threshold execute successfully without requiring manual command syntax.
- **SC-002**: 100% of destructive or remote intents log a user-visible confirmation (with event id and undo instructions) before execution.
- **SC-003**: At least 90% of ambiguous utterances result in clarifying prompts or contextual suggestions instead of hard errors.
- **SC-004**: Intent detection and confirmation logging enables regeneration of chat-driven workflows such that replaying orchestration events reproduces the same sequence for 100% of recorded sessions sampled during acceptance testing.
- **SC-005**: Contextual suggestion latency remains under 2 seconds for Bases with up to 500 orchestration events, ensuring conversational responsiveness.

