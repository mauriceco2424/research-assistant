# Tasks: Chat Assistant & Intent Routing

**Input**: Design documents from `/specs/006-chat-intent-routing/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Integration tests focus on the chat-session harness; no contract tests required.

**Organization**: Tasks are grouped by user story to keep each slice independently deliverable.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare workspace folders and scaffolding needed by all phases.

- [ ] T001 Create intent module scaffolding under `src/orchestration/intent/` (payload.rs, confirmation.rs, registry.rs, log.rs)
- [ ] T002 Add new `intents` directory initialization to `src/bases/layout.rs` so each Base gets `/AI/<Base>/intents`
- [ ] T003 [P] Update integration harness registry in `tests/integration/mod.rs` to include forthcoming intent router tests

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure required before any user story work.

- [ ] T004 Define `IntentPayload` structure plus serde helpers in `src/orchestration/intent/payload.rs`
- [ ] T005 Implement `ConfirmationTicket` model + file persistence in `src/orchestration/intent/confirmation.rs`
- [ ] T006 Extend orchestration logging with `intent_detected/confirmed/executed/failed` enums in `src/orchestration/events.rs`
- [ ] T007 [P] Create AI-layer JSONL writer/reader for intents in `src/orchestration/intent/log.rs`
- [ ] T008 Add capability registry skeleton (`CapabilityDescriptor`, registration API) in `src/orchestration/intent/registry.rs`
- [ ] T009 Implement global remote-disable guard in `src/chat/intent_router/dispatcher.rs` (block `safety_class = remote` when toggle is off)

**Checkpoint**: Intent schema, confirmation persistence, remote toggles, and logging primitives ready.

---

## Phase 3: User Story 1 - Natural chat → command execution (Priority: P1) – MVP

**Goal**: Parse multi-intent chat messages, queue them, stop execution when earlier intents fail, and execute underlying commands with orchestration logging.

**Independent Test**: Use integration harness to send “Summarize the last 3 papers and show my writing profile” (success) and a variant where the first intent fails; verify event IDs and cancellation behavior.

### Implementation

- [ ] T010 [P] [US1] Implement NLP parsing + segmentation logic in `src/chat/intent_router/parser.rs`
- [ ] T011 [P] [US1] Build intent queue + dispatcher sequencing in `src/chat/intent_router/dispatcher.rs`
- [ ] T012 [US1] Add failure short-circuit/cancellation handling to dispatcher in `src/chat/intent_router/dispatcher.rs`
- [ ] T013 [US1] Wire router into chat session entrypoint (`src/chat/mod.rs`) to intercept messages before existing command handlers
- [ ] T014 [US1] Add execution feedback + orchestration status responses to `src/chat/commands/mod.rs`
- [ ] T015 [P] [US1] Extend chat-session integration harness with `tests/integration/intent_router_execute.rs` covering multi-intent success flow
- [ ] T016 [P] [US1] Add failure/cancellation integration test in `tests/integration/intent_router_failure.rs`

**Checkpoint**: User can issue multi-intent instructions; assistant executes sequentially with logs and cancels downstream intents when needed.

---

## Phase 4: User Story 2 - Safety gating & clarifications (Priority: P2)

**Goal**: Enforce confirmations for destructive/remote intents, prompt for missing parameters at confidence <0.80, and guide users when no Base is active.

**Independent Test**: Issue “delete the profile” and verify clarification card, confirm phrase enforcement, and event logging; disable Base selection and confirm router replies with recovery instructions.

### Implementation

- [ ] T017 [P] [US2] Implement safety classifier + confidence threshold checks in `src/chat/intent_router/safety.rs`
- [ ] T018 [US2] Surface clarification prompts + option lists in chat reply builder (`src/chat/intent_router/ui.rs`)
- [ ] T019 [US2] Connect confirmation ticket workflow to actual command execution (write/read tickets) in `src/chat/intent_router/confirmation_flow.rs`
- [ ] T020 [US2] Ensure remote inference prompts display manifest summaries and consent reminders in `src/chat/commands/mod.rs`
- [ ] T021 [US2] Add Base/offline guard before routing in `src/chat/intent_router/dispatcher.rs` (notify user to select a Base)
- [ ] T022 [P] [US2] Add destructive-action + missing-Base integration tests in `tests/integration/intent_router_confirmation.rs`

**Checkpoint**: Destructive/remote commands always confirm; low-confidence or context-missing intents ask clarifying questions instead of guessing.

---

## Phase 5: User Story 3 - Context-aware suggestions & fallback (Priority: P3)

**Goal**: Provide proactive suggestions based on AI-layer state and graceful fallback when intents cannot be routed.

**Independent Test**: Seed pending consent + stale knowledge entries, ask “What should I do next?” and verify assistant surfaces evidence-backed suggestions; ambiguous command shows fallback instructions.

### Implementation

- [ ] T023 [P] [US3] Implement `SuggestionContext` snapshot builder in `src/orchestration/intent/suggestions.rs`
- [ ] T024 [US3] Hook suggestion engine into chat loop (`src/chat/intent_router/suggestions.rs`) with evidence citations
- [ ] T025 [US3] Implement fallback guidance + manual-command hints for failed intents in `src/chat/intent_router/fallback.rs`
- [ ] T026 [P] [US3] Integration test for contextual suggestion + fallback flows in `tests/integration/intent_router_suggestions.rs`

**Checkpoint**: Assistant remains helpful even when unsure, citing evidence for suggestions.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final documentation, benchmarks, and validation.

- [ ] T027 Update quickstart walkthrough with final screenshots/CLI snippets in `specs/006-chat-intent-routing/quickstart.md`
- [ ] T028 Add developer docs for capability registration and router troubleshooting in `docs/intent-router.md` and reference in `docs/CHANGELOG.md`
- [ ] T029 Benchmark intent routing latency on Bases with 5/50/500 events; document results in `docs/perf/intent_router.md`
- [ ] T030 [P] Run full `cargo test` + manual chat validation, document results in `tests/README.md`
- [ ] T031 Review intent logs for P1/P2 compliance, add troubleshooting tips to `docs/intent-router.md`
- [ ] T032 [P] Final lint/format pass across new Rust modules (`cargo fmt`, `cargo clippy`)

---

## Dependencies & Execution Order

1. **Phase 1 → Phase 2**: Setup scaffolding before defining core models.
2. **Phase 2 → User Stories**: All stories depend on payloads, confirmation storage, logging, remote toggles, and registry.
3. **User Story Order**: US1 (MVP) → US2 (safety/clarification) → US3 (suggestions). Later stories may start once Foundational phase completes but sequencing reduces merge conflicts.
4. **Polish**: Runs after desired user stories are done.

### Parallel Execution Examples

- Foundational tasks T004–T009 touch different files and can proceed concurrently once scaffolding exists.
- US1 parsing (T010) and dispatcher (T011) can run in parallel, converging at T013; failure handling (T012) can overlap once queue scaffold exists.
- US2 safety classifier (T017) and confirmation-flow wiring (T019) can run concurrently; Base guard (T021) operates in dispatcher file and should merge after T012.
- US3 suggestion builder (T023) and fallback handler (T025) can proceed simultaneously; both depend on payload schema.

## Implementation Strategy

1. Complete Phases 1–2 to establish shared infrastructure.
2. Deliver US1 as MVP (multi-intent routing + logging + failure cancellation).
3. Layer US2 safety/consent/Base guarding to enforce constitutional guarantees.
4. Add US3 contextual suggestions/fallback to improve UX.
5. Finish with polish tasks (docs, benchmarks, validation, lint).
