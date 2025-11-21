---

description: "Tasks for Paper Discovery Consent Workflow"
---

# Tasks: Paper Discovery Consent Workflow

**Input**: Design documents from `/specs/009-paper-discovery/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Not explicitly requested in spec; focus on implementation and observable outcomes. UI work is deferred; use backend/chat-response stubs instead.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Ensure dependencies and config ready for discovery/acquisition work.

- [ ] T001 [P] Verify/add discovery-needed crates (reqwest/serde/tokio features) in Cargo.toml
- [ ] T002 Ensure prompt manifest/config slots exist for discovery operations in src/api/config/mod.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared models, storage, orchestration, and dedup utilities needed by all stories.

- [X] T003 Define discovery data structs (CandidatePaper, ApprovalBatch, AcquisitionEvent, StoredPaperRecord) in src/models/discovery.rs
- [X] T004 Implement dedup utility (DOI/arXiv/eprint, fallback title+first-author+year) in src/services/dedup.rs
- [X] T005 Add orchestration/consent event schema for discovery batches in src/models/orchestration/discovery.rs
- [X] T006 Wire prompt manifest logging + endpoint capture for discovery/approval AI calls in src/services/ai/manifest.rs
- [X] T007 Implement Base storage helpers for metadata/PDF paths and NEEDS_PDF flagging in src/services/storage/discovery.rs
- [X] T008 Add chat/text response helpers for consent prompts and outcome summaries in src/api/discovery.rs (backend-only)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel.

---

## Phase 3: User Story 1 - Topic Discovery with Consent (Priority: P1) - MVP

**Goal**: Topic-based discovery in chat with batch approval, consent logging, acquisition, and NEEDS_PDF handling.

**Independent Test**: From chat, request a topic, receive candidates, approve subset with chosen acquisition mode, see outcomes logged and NEEDS_PDF flagged without affecting gap/session flows.

### Implementation for User Story 1

- [X] T009 [US1] Implement POST /discovery/requests (topic mode) per contract in src/api/discovery.rs
- [X] T010 [P] [US1] Build topic candidate generation/ranking with manifest annotation in src/services/discovery/topic.rs
- [X] T011 [US1] Persist request + candidates to AI-layer storage in src/services/discovery/store.rs
- [X] T012 [US1] Implement POST /discovery/approvals with consent logging and batch creation in src/api/discovery.rs
- [X] T013 [US1] Implement acquisition worker for approved batches (metadata/PDF, NEEDS_PDF on failure) in src/services/acquisition/discovery.rs
- [X] T014 [US1] Add chat command/handler for topic discovery request + approval prompts (backend-only) in src/chat/mod.rs
- [ ] T015 [US1] Add chat outcome summary rendering (successes, NEEDS_PDF, skips/dupes) in src/chat/mod.rs

**Checkpoint**: Topic discovery MVP functional and testable independently.

---

## Phase 4: User Story 2 - KnowledgeProfile Gap Filling (Priority: P2)

**Goal**: Gap-driven discovery proposing candidates tied to KnowledgeProfile weaknesses with consented acquisition.

**Independent Test**: Trigger gap discovery, see candidates annotated with gap rationale, approve, and verify stored records carry gap provenance.

### Implementation for User Story 2

- [ ] T016 [US2] Extend discovery request handling for gap mode using KnowledgeProfile inputs in src/services/discovery/gap.rs
- [X] T017 [P] [US2] Render gap rationale and approval prompts via chat response builder in src/chat/mod.rs
- [ ] T018 [US2] Persist gap provenance into StoredPaperRecord/provenance in src/services/discovery/store.rs

**Checkpoint**: Gap-based discovery works independently and preserves provenance.

---

## Phase 5: User Story 3 - Session Follow-ups (Priority: P3)

**Goal**: Session-context follow-up discovery with consented acquisition and provenance linking to prior sessions.

**Independent Test**: From a recent session, request follow-ups, approve batch, and see outcomes logged with session reference.

### Implementation for User Story 3

- [ ] T019 [US3] Extend discovery request handling for session mode using recent session context in src/services/discovery/session.rs
- [X] T020 [P] [US3] Add chat response for session follow-up prompts/approvals in src/chat/mod.rs
- [ ] T021 [US3] Record session reference in orchestration/provenance logs in src/models/orchestration/discovery.rs

**Checkpoint**: Session follow-up discovery operates independently with correct provenance.

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, resilience, and consistency across stories.

- [ ] T022 [P] Update quickstart validations for discovery flows in specs/009-paper-discovery/quickstart.md
- [ ] T023 [P] Add runbook for offline/remote-AI-unavailable scenarios in docs/runbooks/discovery.md
- [ ] T024 Final audit: dedup decisions + consent/orchestration logs verified across discovery/acquisition in src/services/discovery

---

## Phase N+1: Success Criteria Validation

**Purpose**: Validate measurable outcomes for latency, manifests, provenance, and NEEDS_PDF handling.

- [X] T025 [P] Add latency check for SC-001 (topic/gap/session request -> candidates under 30s) in tests/integration/discovery_latency.rs
- [X] T026 [P] Add manifest/network audit for SC-005 (prompt manifests + endpoints logged; no hidden calls) in tests/integration/discovery_audit.rs
- [X] T027 Validate provenance persistence for SC-003 (approved items persisted with consent/provenance) in tests/integration/discovery_provenance.rs
- [X] T028 Validate NEEDS_PDF handling for SC-004 (failed fetch -> metadata-only + reason) in tests/integration/discovery_needs_pdf.rs

---

## Dependencies & Execution Order

- Setup -> Foundational -> User stories (US1 -> US2 -> US3) -> Polish -> Success validation.
- Within stories: API handlers depend on foundational models/storage; UI tasks depend on backend endpoints; acquisition worker depends on approval creation.

## Parallel Execution Examples

- After Foundational: T010 (candidate generation) can run in parallel with T011 (storage) and T014 (chat handler) once their prerequisites are ready.
- US2 chat response (T017) can proceed in parallel with backend gap handler (T016) after foundational models.
- US3 chat response (T020) can proceed in parallel with session handler (T019) after foundational models.

## Implementation Strategy

- MVP: Complete Setup + Foundational + US1; validate topic discovery end-to-end before proceeding.
- Incremental: Add US2 (gap) then US3 (session) as independent increments, keeping chat-first consent and provenance intact.
