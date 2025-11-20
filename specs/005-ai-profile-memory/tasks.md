# Tasks: AI Profiles & Long-Term Memory

**Input**: Design documents from `/specs/005-ai-profile-memory/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Integration tests are included where they directly validate each user story's flows (chat commands, consent logging, regeneration). Unit tests are embedded within implementation tasks when they are inseparable from the code being written.

**Organization**: Tasks are grouped by user story so each slice is independently implementable and testable.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish module scaffolding and routing hooks required by all subsequent phases.

- [X] T001 Create `src/orchestration/profiles/mod.rs` scaffolding with module exports for model, storage, service, interview, render, governance.
- [X] T002 [P] Introduce profile directory/path constants in `src/bases/layout.rs` for `/AI/<Base>/profiles` and `/User/<Base>/profiles`.
- [X] T003 [P] Register `profile *` chat command stubs in `src/chat/commands/mod.rs` delegating to a new `profile.rs` handler file.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure (data structures, storage, consent, migrations) that every user story depends on.

- [X] T004 Define profile data structures (`ProfileMetadata`, `ProfileEntry`, `ProfileChangeEvent`, `ConsentManifest`) in `src/orchestration/profiles/model.rs` per data-model.md.
- [X] T005 Implement deterministic JSON/HTML storage helpers (read/write/hash) in `src/orchestration/profiles/storage.rs`.
- [X] T006 Extend orchestration logging to capture profile change payloads, hashes, and undo tokens in `src/orchestration/events.rs`.
- [X] T007 Build consent manifest repository with read/write APIs in `src/orchestration/consent/store.rs`.
- [X] T008 Add Base migration to seed empty profile shells plus metadata in `src/bases/migrations/profile_shells.rs`.
- [X] T009 Implement profile scope configuration and enforcement utilities in `src/orchestration/profiles/scope.rs`.
- [X] T010 [P] Create reusable integration test fixture for sandbox Bases under `tests/integration/support/profile_base.rs`.

**Checkpoint**: Storage, logging, consent, and migrations ready — user stories can start.

---

## Phase 3: User Story 1 - Profile inspection & inline edits (Priority: P1) **MVP**

**Goal**: Allow researchers to run `profile show`/`profile update` from chat, view structured summaries, and edit fields with confirmation + audit trail.

**Independent Test**: From a seeded Base, run show/update commands for each profile type and verify chat output includes summaries, timestamps, evidence references, and the logged event ID while the JSON + HTML artifacts update locally.

### Implementation

- [X] T011 [P] [US1] Add integration test covering `profile show` + `profile update` happy path in `tests/integration/profile_show_update.rs`.
- [X] T012 [US1] Build summary + HTML renderers (JSON narrative + attachment) in `src/orchestration/profiles/summarize.rs`.
- [X] T013 [US1] Implement `profile show <type>` command routing + response formatting in `src/chat/commands/profile.rs`.
- [X] T014 [US1] Implement inline edit workflow (diff preview, confirmation, orchestration event write) in `src/orchestration/profiles/service.rs`.
- [X] T015 [US1] Generate HTML summaries and attach file metadata for chat responses in `src/orchestration/profiles/render.rs`.
- [X] T016 [US1] Enforce scope flags when returning profile data via shared API hooks in `src/orchestration/profiles/api.rs`.
- [X] T016A [US1] Implement and expose `profile.get_work_context()` API in `src/orchestration/profiles/api.rs` with scope-aware filtering.
- [X] T016B [US1] Handle missing profile files by seeding default scaffolds and "uninitialized" markers in `src/orchestration/profiles/service.rs`; extend integration test to cover this flow.

**Checkpoint**: Users can inspect and edit long-term profiles entirely from chat; audits and file artifacts stay in sync.

---

## Phase 4: User Story 2 - Guided interviews & capture runs (Priority: P2)

**Goal**: Deliver guided interviews (`profile interview/run`) with consent manifests, remote inference approvals, partial saves, and confirmation before overwriting.

**Independent Test**: Trigger interview commands with and without remote inference, confirm consent manifests are requested/logged, partial entries tagged when consent denied, and final acceptance overwrites JSON safely.

### Implementation

- [X] T017 [P] [US2] Add integration test simulating consent approval/denial during `profile interview` in `tests/integration/profile_interview.rs`.
- [X] T018 [US2] Implement interview orchestration state machine (question flow, partial saves) in `src/orchestration/profiles/interview.rs`.
- [X] T019 [US2] Build prompt manifest generator + consent logging linkages in `src/orchestration/consent/manifest.rs`.
- [X] T020 [US2] Extend `src/chat/commands/profile.rs` with `profile interview`/`profile run writing-style` handlers including confirmation prompts.
- [X] T021 [US2] Persist interview outcomes (source attribution, consent metadata, `NEEDS_REMOTE_APPROVAL` flags) in `src/orchestration/profiles/service.rs`.
- [X] T021A [US2] Instrument interview completion metrics (local counters + confirmation logs) and extend integration coverage in `tests/integration/profile_interview.rs`.

**Checkpoint**: Interview flows capture fresh data with explicit consent handling and overwrite protection.

---

## Phase 5: User Story 3 - Knowledge readiness for Learning Mode (Priority: P3)

**Goal**: Maintain KnowledgeProfile mastery records with evidence links, weakness tags, and provide summary APIs for Learning Mode/learning readiness checks.

**Independent Test**: Using populated knowledge entries, adjust mastery/weaknesses via chat, ensure evidence linkage + stale detection works, and retrieve `profile.get_knowledge_summary()` JSON for Learning Mode without touching other stories.

### Implementation

- [X] T022 [P] [US3] Create integration test covering knowledge strengths/weakness toggles and summary API in `tests/integration/profile_knowledge_ready.rs`.
- [X] T023 [US3] Implement KnowledgeProfile entry mutations (strength, weakness, review time) in `src/orchestration/profiles/knowledge.rs`.
- [X] T024 [US3] Build evidence linking + stale detection against papers/notes in `src/orchestration/profiles/linking.rs`.
- [X] T025 [US3] Implement `profile.get_knowledge_summary()` response builder in `src/orchestration/profiles/api.rs`.
- [X] T026 [US3] Expose knowledge summary hook to Learning Mode orchestrations in `src/orchestration/learning/interface.rs`.

**Checkpoint**: Knowledge readiness surfaces concept mastery data with evidence integrity for Learning Mode.

---

## Phase 6: User Story 4 - Profile governance, audit, and regenerability (Priority: P4)

**Goal**: Provide governance tools (`profile audit/export/delete/regenerate/scope`) that preserve local-first guarantees and deterministic recovery from orchestration history.

**Independent Test**: Run each governance command independently on seeded data; verify audit output lists events + undo tokens, exports produce ZIP/HTML packages, deletes respect scope, and regenerate halts on hash mismatch.

### Implementation

- [X] T027 [P] [US4] Add integration test for audit/export/delete/regenerate flows in `tests/integration/profile_governance.rs`.
- [X] T028 [US4] Implement `profile audit` chat flow showing chronological events and undo instructions in `src/chat/commands/profile.rs`.
- [X] T029 [US4] Implement export + delete handlers (ZIP creation, scope-aware deletes) in `src/orchestration/profiles/governance.rs`.
- [X] T029A [US4] Add filesystem lock/queue handling for exports and cover concurrent export/write scenarios in `tests/integration/profile_governance.rs`.
- [X] T030 [US4] Implement deterministic regeneration with hash verification + failure guidance in `src/orchestration/profiles/regenerate.rs`.
- [X] T031 [US4] Implement `profile scope` command (set/list) syncing with scope utilities in `src/chat/commands/profile.rs`.

**Checkpoint**: Users can audit, export, delete, and regenerate profiles confidently with documented undo steps.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Finalize docs, quickstart validation, and ensure orchestration transparency across the feature.

- [X] T032 [P] Update Quickstart walkthroughs + troubleshooting for interviews/governance in `specs/005-ai-profile-memory/quickstart.md`.
- [X] T033 Capture architecture + consent updates in `docs/CHANGELOG.md` or equivalent release notes (create file if missing).
- [X] T034 Run full `cargo test` + manual quickstart validation documenting results in `tests/README.md`.
- [X] T035 Benchmark `profile show` latency across sample Bases (5/50/500 entries) and record results vs. SC-001 in `docs/perf/profile_show.md`.

---

## Dependencies & Execution Order

1. **Setup (Phase 1)** → 2. **Foundational (Phase 2)**: All later work depends on storage, logging, consent, and migrations being in place.  
3. **User Story 1 (Phase 3)**: First deliverable (MVP) depends only on phases 1-2.  
4. **User Story 2 (Phase 4)**: Requires interview infrastructure plus US1 artifacts for shared profile services.  
5. **User Story 3 (Phase 5)**: Depends on foundational storage/API plus US1 summary APIs for consistent data access.  
6. **User Story 4 (Phase 6)**: Depends on phases 1-2 and benefits from event logging built earlier, but remains independent of US2/US3 behaviors.  
7. **Polish (Phase 7)**: Runs after chosen user stories are complete.

### Story Dependency Graph

`Setup → Foundational → (US1 MVP) → (US2 || US3) → US4 → Polish`

---

## Parallel Execution Examples

- **US1**: T011 (integration test) can run parallel with T012 summary builder once foundational code exists; T013/T014 should follow sequentially to avoid command conflicts.
- **US2**: T018 interview orchestration and T019 manifest builder can progress concurrently because they touch different files; T020 depends on both.
- **US3**: T023 knowledge mutations and T024 evidence linking operate on different modules and can proceed in parallel before T025 aggregates outputs.
- **US4**: T028 audit command and T029 export/delete can be parallelized while T030 regeneration waits on event logging readiness.

---

## Implementation Strategy

1. **MVP First**: Complete Phases 1-3 so `profile show/update` works end-to-end; demo and validate via integration test T011.  
2. **Incremental Delivery**: Ship US2 interviews next, then US3 knowledge readiness, followed by US4 governance. Each story has its own integration test to confirm readiness before progressing.  
3. **Parallel Workstreams**: After Foundational tasks, separate contributors can own US2 and US3 simultaneously while another focuses on US4 prep, as long as they respect shared file boundaries called out in the tasks.  
4. **Validation**: After each story, run its dedicated integration test plus `cargo test`; finish with Phase 7 to document and re-validate the entire profile lifecycle.
