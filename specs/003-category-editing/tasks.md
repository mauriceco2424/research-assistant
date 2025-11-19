---

description: "Tasks for categorization & editing workflows implementation"
---

# Tasks: Categorization & Editing Workflows

**Input**: Design documents from `/specs/003-category-editing/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: No explicit TDD requirement. Integration tests are added where they provide independent verification per user story.

**Organization**: Tasks are grouped by user story so each story can be implemented and tested independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Task can run in parallel (different files, no dependencies)
- **[Story]**: User story label (US1, US2, US3, US4) for story phases
- Paths use the single-project structure described in plan.md

## Path Conventions

- Source: `src/`
- Tests: `tests/`
- Documentation: `specs/003-category-editing/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare shared configuration and storage locations needed by all user stories.

- [X] T001 Create AI-layer `categories/` directories (definitions, assignments, snapshots) in `src/bases/fs_paths.rs`.
- [X] T002 Add category proposal tunables (`categorization.max_proposals`, `categorization.timeout_ms`) to `src/bases/config.rs`.
- [X] T003 [P] Document new configuration knobs and directory layout in `specs/003-category-editing/quickstart.md`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core persistence, orchestration, and report plumbing required before any user story work.

- [X] T004 Implement `CategoryStore` (load/save CategoryDefinition + Narrative) in `src/bases/categories.rs`.
- [X] T005 Implement `CategoryAssignmentsIndex` with incremental updates in `src/bases/categories.rs`.
- [X] T006 Implement `CategorySnapshotStore` (capture/restore JSON snapshots) in `src/bases/categories_snapshot.rs`.
- [X] T007 Extend `OrchestrationLog` with category edit event types & payload helpers in `src/orchestration/mod.rs`.
- [X] T008 Wire report regeneration entry point to consume category definitions/pins in `src/reports/mod.rs`.
- [X] T009 Add consent manifest plumbing for `category_narrative_suggest` operations in `src/orchestration/consent.rs`.
- [X] T010 [P] Add contract skeleton `specs/003-category-editing/contracts/categories.yaml` to the build/test harness list in `tests/integration/mod.rs`.

**Checkpoint**: Persistence, orchestration, and report hooks are ready. User-story phases can run in parallel.

---

## Phase 3: User Story 1 – AI-Proposed Categories (Priority: P1) ✅ MVP

**Goal**: Generate AI-assisted category proposals with confidence scores, let researchers accept or edit them via chat, and persist accepted structures with report regeneration.

**Independent Test**: On a Base with ≥50 papers, run `categories propose`, inspect preview cards, accept/rename a subset, and verify reports regenerate plus orchestration logs record the actions without touching other stories.

### Implementation

- [X] T011 [P] [US1] Build `FeatureVectorBuilder` (TF-IDF + embeddings) to feed clustering in `src/reports/categorization/features.rs`.
- [X] T012 [US1] Implement local clustering + scoring pipeline (`categories propose` worker) in `src/reports/categorization/proposals.rs`.
- [X] T013 [US1] Persist proposal previews (definitions + confidence) under `AI/<Base>/categories/proposals/*.json` in `src/bases/categories.rs`.
- [X] T014 [US1] Implement chat command `categories propose` + preview rendering in `src/chat/mod.rs`.
- [X] T015 [US1] Implement chat/API handler for applying/renaming/rejecting proposals per `contracts/categories.yaml` in `src/chat/api/categories.rs`.
- [X] T016 [US1] Log proposal + acceptance events (including consent manifest references) in `src/orchestration/mod.rs`.
- [X] T017 [US1] Regenerate HTML category/global reports after acceptance in `src/reports/mod.rs`.
- [X] T018 [US1] Add integration test `tests/integration/categories_proposals.rs` covering propose → accept flow and report regeneration.

**Checkpoint**: Researchers can generate and apply proposals entirely via chat; reports stay in sync. This is the MVP slice.

---

## Phase 4: User Story 2 – Chat-Based Category Editing (Priority: P1)

**Goal**: Provide rename, merge, split, move, and undo operations with orchestration logging and reversible snapshots.

**Independent Test**: From a Base with existing categories, rename one, merge two others, split an overloaded category, move selected papers, and run `category undo` to roll back the last change—all without invoking later stories.

### Implementation

- [ ] T019 [P] [US2] Implement rename + slug update helpers in `src/bases/categories.rs`.
- [ ] T020 [US2] Implement merge pipeline (merge assignments, pins, narratives) in `src/bases/categories_merge.rs`.
- [ ] T021 [US2] Implement split suggestion engine + confirmation handling in `src/reports/categorization/split.rs`.
- [ ] T022 [US2] Implement move/bulk assignment API in `src/bases/categories_assignments.rs`.
- [ ] T023 [US2] Capture per-edit snapshots + single-step undo command in `src/orchestration/mod.rs`.
- [ ] T024 [US2] Add chat commands `category rename/merge/split/move/undo` in `src/chat/mod.rs`.
- [ ] T025 [US2] Update contracts for rename/merge/split/move endpoints in `specs/003-category-editing/contracts/categories.yaml`.
- [ ] T026 [US2] Add integration test `tests/integration/categories_editing.rs` covering rename → merge → undo flow.

**Checkpoint**: Full editing toolbox with undo is available via chat/API, independently testable.

---

## Phase 5: User Story 3 – Category Health & Backlog View (Priority: P2)

**Goal**: Surface backlog metrics, pinned highlights, staleness, and overloaded categories via `categories status`.

**Independent Test**: With ≥10 categories and backlog of 30 uncategorized papers, run `categories status` to see counts, stale indicators, backlog segments, and actionable recommendations without relying on US4 features.

### Implementation

- [ ] T027 [P] [US3] Implement `CategoryMetricsCollector` aggregating counts, staleness, overload ratios in `src/reports/categorization/status.rs`.
- [ ] T028 [US3] Detect uncategorized backlog segments (topic inference + counts) in `src/reports/categorization/status.rs`.
- [ ] T029 [US3] Persist latest metrics for history/reference in `src/bases/categories_metrics.rs`.
- [ ] T030 [US3] Implement chat command `categories status` with actionable recommendations in `src/chat/mod.rs`.
- [ ] T031 [US3] Extend contracts with `/categories/status` endpoint schema in `specs/003-category-editing/contracts/categories.yaml`.
- [ ] T032 [US3] Add integration test `tests/integration/categories_status.rs` ensuring backlog and overload alerts render correctly.

**Checkpoint**: Researchers can independently inspect category health/backlog and act on recommendations.

---

## Phase 6: User Story 4 – Narrative Editing & Report Sync (Priority: P2)

**Goal**: Let researchers update narratives, learning prompts, pinned papers, and figure gallery toggles via chat, keeping AI-layer and HTML reports synchronized with consent logging.

**Independent Test**: Edit a category narrative, pin/unpin papers, toggle figure galleries, regenerate reports, and verify the changes persist along with consent manifests (if AI assistance used) without invoking other stories.

### Implementation

- [ ] T033 [P] [US4] Implement narrative diff/apply helpers (summary, prompts, notes) in `src/bases/categories_narrative.rs`.
- [ ] T034 [US4] Add optional AI-assist flow with consent prompt and manifest write in `src/chat/mod.rs` + `src/orchestration/consent.rs`.
- [ ] T035 [US4] Update pinned-paper ordering + figure gallery toggle persistence in `src/bases/categories.rs`.
- [ ] T036 [US4] Update HTML report rendering to reflect narratives, pins, and gallery flags in `src/reports/mod.rs`.
- [ ] T037 [US4] Implement chat command `category narrative` (edit + preview) and `category pin` shortcuts in `src/chat/mod.rs`.
- [ ] T038 [US4] Add integration test `tests/integration/categories_narratives.rs` ensuring edits propagate to reports and consent logs.

**Checkpoint**: Narrative and pin management works end-to-end with regeneration and consent handling.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T039 [P] Harden error messaging & conflict detection for concurrent edits in `src/chat/mod.rs`.
- [ ] T040 Document health metrics + undo usage in `specs/003-category-editing/quickstart.md`.
- [ ] T041 Add telemetry/metrics entries for proposal/undo durations in `src/orchestration/mod.rs`.
- [ ] T042 Run end-to-end regression script (us1→us4) and capture notes in `specs/003-category-editing/research.md`.

---

## Dependencies & Execution Order

- Setup (Phase 1) → Foundational (Phase 2) → User stories (Phases 3–6) → Polish (Phase 7).
- User stories depend on Foundational artifacts but can proceed in parallel once Phase 2 completes.

### User Story Dependencies

1. **US1 (P1)**: Unlocks core categorization data and is the MVP slice.
2. **US2 (P1)**: Depends on US1 persistence hooks for definitions/assignments.
3. **US3 (P2)**: Depends on US1 assignments to compute metrics but independent of US2 editing flows.
4. **US4 (P2)**: Depends on US1 narratives + report hooks; can proceed alongside US3.

### Parallel Opportunities

- Setup tasks T001–T003 can run concurrently.
- Foundational tasks T004–T010 touch different modules and can be parallelized where no file overlap exists.
- After Phase 2, teams can split across US1–US4 as noted; each story’s `[P]` tasks indicate safe parallelization.

---

## Implementation Strategy

1. Complete Phases 1–2 to establish storage, orchestration, and report plumbing.
2. Ship MVP by finishing Phase 3 (US1) and validating proposals + acceptance end-to-end.
3. Layer on US2 editing controls, then US3 status views, followed by US4 narrative sync.
4. Finish with Polish tasks to tighten UX, docs, and telemetry before handing off to `/speckit.implement`.
