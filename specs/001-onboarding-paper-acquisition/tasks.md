---

description: "Tasks for onboarding & paper acquisition workflow implementation"
---

# Tasks: Onboarding & Paper Acquisition Workflow

**Input**: Design documents from `/specs/001-onboarding-paper-acquisition/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: No explicit TDD requirement in the feature spec. Test tasks are omitted here and can be added later if requested.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Single project: `src/`, `tests/` at repository root
- This feature uses:
  - `src/chat/`, `src/bases/`, `src/ingestion/`, `src/acquisition/`, `src/reports/`, `src/orchestration/`
  - `tests/integration/`, `tests/unit/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Create base source tree for ResearchBase in `src/` (directories: `src/chat/`, `src/bases/`, `src/ingestion/`, `src/acquisition/`, `src/reports/`, `src/orchestration/`)
- [X] T002 [P] Create initial test directories for this feature in `tests/integration/` and `tests/unit/`
- [X] T003 Define configuration location and format for per-install settings in `src/bases/` (including `last_active_base_id` and acquisition preferences)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Define Paper Base entity representation and persistence helpers in `src/bases/` (fields: id, name, user_layer_path, ai_layer_path, created_at, last_active_at, settings)
- [X] T005 Define Library Entry entity representation and persistence helpers in `src/bases/` (fields: entry_id, base_id, title, authors, venue, year, identifier, pdf_paths, needs_pdf, timestamps)
- [X] T006 Define Acquisition Batch and Orchestration Event entity representations in `src/orchestration/` (including batch_id, event_id, payload structure, related_batch_id)
- [X] T007 Implement filesystem layout conventions for User Layer and AI Layer per Base in `src/bases/` (directory naming and creation rules)
- [X] T008 Implement base-level orchestration logging utilities in `src/orchestration/` (append-only event records stored in AI Layer)
- [X] T009 Implement skeleton chat intent routing hooks for onboarding and acquisition commands in `src/chat/` (intents for Path A, Path B, discovery, history, and undo)

**Checkpoint**: Foundation ready – Base storage, entities, orchestration, and chat hooks exist for all stories to build on.

---

## Phase 3: User Story 1 - Create and Select Paper Bases (Priority: P1) – MVP

**Goal**: Allow users to create multiple Paper Bases, persist their User/AI layers, and select an active Base at startup and via chat.

**Independent Test**: From a clean install, user can create at least one Base, see it in the Base list, select it as active, restart, and confirm the previously active Base is remembered and can be changed.

### Implementation for User Story 1

- [X] T010 [US1] Implement Base creation command handler in `src/chat/` that collects a Base name and triggers Base creation in `src/bases/`
- [X] T011 [US1] Implement Base creation logic in `src/bases/` that creates User Layer and AI Layer directories on disk and records a new Paper Base entity
- [X] T012 [US1] Implement Base listing command handler in `src/chat/` that lists available Bases from storage in `src/bases/`
- [X] T013 [US1] Implement Base selection logic in `src/bases/` that marks a Base as active and updates `last_active_base_id` in configuration
- [X] T014 [US1] Implement startup logic in `src/bases/` that loads `last_active_base_id` and verifies both User Layer and AI Layer directories exist (creating them if missing)
- [X] T015 [US1] Implement an orchestration event emission in `src/orchestration/` for Base creation and Base selection events
- [X] T016 [US1] Implement chat feedback messages in `src/chat/` for Base creation, Base listing, and active Base confirmation

**Checkpoint**: User can create, list, select, and persist Bases, and the system records these operations as orchestration events.

---

## Phase 4: User Story 2 - Onboarding with Existing PDFs (Path A) (Priority: P1)

**Goal**: Let users ingest existing PDFs or exports into the active Base, produce initial categories, and generate HTML reports without downloading new PDFs.

**Independent Test**: Starting from an active Base with no papers, user completes a Path A flow and ends with a populated library, initial categories, and at least one category report and one global report.

### Implementation for User Story 2

- [X] T017 [US2] Implement Path A onboarding command handler in `src/chat/` that lets the user indicate they have PDFs/exports and provides a path or selection mechanism
- [X] T018 [US2] Implement local PDF and export ingestion pipeline in `src/ingestion/` that scans the provided path, extracts metadata/text, and creates Library Entries without network calls
- [X] T019 [US2] Implement AI-layer updates in `src/ingestion/` to record ingested papers for the active Base in AI Layer files
- [X] T020 [US2] Implement initial category generation logic for Path A in `src/reports/` or `src/bases/` using existing library metadata
- [X] T021 [US2] Implement HTML category report generation in `src/reports/` that writes regenerable reports to the Base's User Layer
- [X] T022 [US2] Implement HTML global report generation in `src/reports/` that summarizes all ingested papers
- [X] T023 [US2] Implement orchestration events in `src/orchestration/` for Path A ingestion and initial report generation
- [X] T024 [US2] Implement chat responses in `src/chat/` that summarize ingestion results and provide links or paths to generated HTML reports

**Checkpoint**: User can ingest local PDFs into an active Base, see a categorized library, and open generated HTML reports without any paper downloads.

---

## Phase 5: User Story 3 - Onboarding without Existing PDFs (Path B) (Priority: P2)

**Goal**: Allow users without local PDFs to describe their research area, have the AI propose candidate papers, and then run a consent-driven acquisition workflow to add selected papers to the Base.

**Independent Test**: Starting from an empty Base, user completes a Path B flow and ends with a small library of candidate papers (some with PDFs, some metadata-only) plus initial categories and reports, with every acquisition step explicitly approved and logged.

### Implementation for User Story 3

- [X] T025 [US3] Implement Path B onboarding interview flow in `src/chat/` that collects research area, questions, and expertise level
- [X] T026 [US3] Implement AI interaction wrapper in `src/chat/` or `src/acquisition/` that uses interview results to request candidate papers (metadata and identifiers only)
- [X] T027 [US3] Implement candidate list presentation and selection in `src/chat/` that lets the user choose which candidates to add and confirms per-batch approval text
- [X] T028 [US3] Implement Paper Candidate representation and staging persistence in `src/acquisition/` based on data-model definitions
- [X] T029 [US3] Implement the Paper Acquisition Workflow execution in `src/acquisition/` that, given an approved batch, performs metadata resolution and open-access PDF retrieval attempts
- [X] T030 [US3] Implement Library Entry creation/update in `src/bases/` for acquired candidates, setting `needs_pdf` when PDFs cannot be retrieved
- [X] T031 [US3] Implement Acquisition Batch and Orchestration Event recording in `src/orchestration/` for Path B acquisition batches (including approval context and per-candidate outcomes)
- [X] T032 [US3] Implement initial categories and HTML report generation for Path B in `src/reports/`, reusing the same mechanisms as Path A where possible
- [X] T033 [US3] Implement chat feedback in `src/chat/` that clearly lists which papers were added with PDFs, which are metadata-only (`NEEDS_PDF`), and what the next steps are

**Checkpoint**: User can onboard without existing PDFs, approve a candidate batch, see a library with mixed PDF/metadata-only entries, and generate reports, with acquisition events logged and consent captured.

---

## Phase 6: User Story 4 - On-Demand Paper Discovery & Acquisition (Priority: P2)

**Goal**: Enable ongoing discovery of new papers for existing Bases using the same Paper Acquisition Workflow as Path B.

**Independent Test**: From a Base with existing papers, user triggers discovery, selects candidates, approves acquisition, and sees new papers in the Base (with PDFs or `NEEDS_PDF` status) and updated reports.

### Implementation for User Story 4

- [X] T034 [US4] Implement discovery commands in `src/chat/` that let users ask for new papers by topic, category gaps, or recent work
- [X] T035 [US4] Implement AI discovery request logic in `src/acquisition/` that retrieves candidate papers for the active Base (metadata and identifiers only)
- [X] T036 [US4] Implement reuse of the candidate selection and approval UI/flow from Path B in `src/chat/` for discovery scenarios
- [X] T037 [US4] Implement reuse of the Paper Acquisition Workflow in `src/acquisition/` for discovery batches, ensuring they are logged separately from onboarding batches
- [X] T038 [US4] Implement updates to Library Entries in `src/bases/` to add discovered papers (PDF-attached or `NEEDS_PDF`) without altering existing entries
- [X] T039 [US4] Implement report regeneration triggers in `src/reports/` so users can easily refresh category/global reports after new papers are acquired
- [X] T040 [US4] Implement chat summaries in `src/chat/` that describe discovery results and how they changed the Base

**Checkpoint**: User can discover and add new papers over time using a single, consistent acquisition workflow, and see the impact in their Base and reports.

---

## Phase 7: User Story 5 - Consent, Logging, and Recovery for Acquisition (Priority: P3)

**Goal**: Give users full visibility into acquisition history and provide the ability to undo at least the last acquisition batch per Base.

**Independent Test**: After one or more acquisition batches, user can view a human-readable acquisition history, identify the context of each batch, and undo the last batch without leaving the Base in an inconsistent state.

### Implementation for User Story 5

- [X] T041 [US5] Implement acquisition history command in `src/chat/` that lists recent Acquisition Batches for the active Base with timestamps and summary stats
- [X] T042 [US5] Implement acquisition history retrieval and formatting in `src/orchestration/` that reads Acquisition Batches and associated Orchestration Events from AI Layer
- [X] T043 [US5] Implement undo-last-batch command in `src/chat/` that asks for confirmation before undoing the most recent batch
- [X] T044 [US5] Implement undo logic in `src/orchestration/` that reverts Library Entries and AI-layer artifacts created solely by the last Acquisition Batch
- [X] T045 [US5] Implement safeguards in `src/orchestration/` to prevent undo operations when they would conflict with later dependent operations (with clear chat error messages)
- [X] T046 [US5] Implement chat feedback in `src/chat/` summarizing what changed as a result of an undo (removed entries, updated reports needed)

**Checkpoint**: Users can inspect acquisition history and safely undo the last batch within a Base, with clear chat feedback and no data corruption.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T047 [P] Add or update developer-facing documentation for onboarding and acquisition flows in `specs/001-onboarding-paper-acquisition/quickstart.md`
- [X] T048 Review and refine chat prompts and responses for all onboarding and acquisition commands in `src/chat/` for clarity and consistency with the constitution
- [X] T049 Improve performance of acquisition-related operations in `src/acquisition/` and `src/orchestration/` for medium-size batches (e.g., up to 100 papers)
- [X] T050 [P] Add basic validation of configuration and filesystem preconditions at app startup in `src/bases/` and `src/orchestration/`
- [X] T051 Review logs and orchestration events in `src/orchestration/` to ensure they contain enough detail for debugging and user audits

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies – can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion – BLOCKS all user stories.
- **User Stories (Phases 3–7)**: All depend on Foundational phase completion.
  - User Story 1 (P1) must be implemented before any onboarding or discovery flows.
  - User Story 2 (P1) and User Story 3 (P2) can start after User Story 1.
  - User Story 4 (P2) depends on User Story 3 for acquisition workflow reuse.
  - User Story 5 (P3) depends on acquisition logging from User Stories 3 and 4.
- **Polish (Phase 8)**: Depends on all desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Foundational; no dependencies on other stories. Must complete before any onboarding or discovery implementation.
- **User Story 2 (P1)**: Depends on Foundational and User Story 1; uses Base management to ingest local PDFs.
- **User Story 3 (P2)**: Depends on Foundational and User Story 1; introduces the Paper Acquisition Workflow used by later stories.
- **User Story 4 (P2)**: Depends on User Story 3; reuses candidate selection and acquisition logic.
- **User Story 5 (P3)**: Depends on acquisition and orchestration events from User Stories 3 and 4.

### Within Each User Story

- Implement entity and persistence logic before orchestration and chat handlers.
- Implement orchestration and logging before undo or history features.
- Ensure each story’s flows are usable and independently testable before starting the next story.

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel.
- Foundational tasks T004–T009 can be parallelized where they touch different files (`src/bases/`, `src/orchestration/`, `src/chat/`).
- Once Foundational and User Story 1 are complete:
  - Path A (User Story 2) and Path B (User Story 3) can progress in parallel.
  - Discovery (User Story 4) can start once the Paper Acquisition Workflow from User Story 3 is in place.
  - Consent/history/undo (User Story 5) can be developed in parallel with late-stage work on discovery, as long as acquisition events are recorded consistently.
- Polish tasks T047 and T050 can run in parallel with minor refinements to chat and orchestration logic.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (critical – blocks all stories).
3. Complete Phase 3: User Story 1 (Base creation and selection).
4. **STOP and VALIDATE**: Ensure Base creation/selection and directory setup work as expected.

At this point, ResearchBase can manage multiple Bases and is ready for onboarding flows.

### Incremental Delivery

1. Deliver MVP (User Story 1).
2. Add User Story 2 (Path A onboarding) – validate ingestion and report generation independently.
3. Add User Story 3 (Path B onboarding with acquisition workflow) – validate candidate → approval → acquisition path and `NEEDS_PDF` handling.
4. Add User Story 4 (on-demand discovery) – validate reuse of acquisition workflow and report regeneration.
5. Add User Story 5 (history and undo) – validate auditability and rollback.

Each increment adds clear user-visible value and builds on the same acquisition and orchestration primitives.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup and Foundational phases together.
2. Once Foundational and User Story 1 are done:
   - Developer A: User Story 2 – local ingestion and initial reports.
   - Developer B: User Story 3 – interview, candidate selection, acquisition workflow.
   - Developer C: User Story 4 – discovery queries and reuse of acquisition workflow.
3. After acquisition and discovery are stable, one developer focuses on User Story 5 – history and undo – while others handle polish and documentation.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] labels map tasks to specific user stories for traceability.
- Each user story should be independently completable and testable at the workflow level.
- Commit after each task or logical group to keep changes small and reviewable.
- Avoid: vague task descriptions, cross-story coupling that breaks independence, or deviations from the consent and local-first constraints in the constitution.
