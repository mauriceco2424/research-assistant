# Tasks: Writing Assistant (LaTeX)

**Input**: Design documents from `/specs/007-writing-assistant/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Functional testing is manual per quickstart; each user story includes clear independent verification steps.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Ensure `src/writing/` module tree exists with placeholders in `src/writing/mod.rs`
- [ ] T002 Add `tectonic` + fallback compiler configuration entries to `src/bin/setup.rs`
- [ ] T003 [P] Add documentation stub in `docs/writing-assistant.md` summarizing project lifecycle commands

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

- [ ] T004 Implement WritingProject manifest loader/saver in `src/writing/project.rs`
- [ ] T005 [P] Extend AI-layer storage helpers in `src/storage/ai_layer.rs` for outline/draft/undo payloads
- [ ] T006 Wire orchestration event enums + logging scaffolds for writing ops in `src/orchestration/events.rs`
- [ ] T007 Create WritingProfile/style model access APIs in `src/profiles/writing_profile.rs`
- [ ] T008 Add chat intent routing stubs for writing commands in `src/chat/intent_routing.rs`

**Checkpoint**: Foundation ready – user story implementation can now begin in parallel.

---

## Phase 3: User Story 1 - Launch Writing Project & Style Interview (Priority: P1) ✅ MVP

**Goal**: Chat command scaffolds a project directory, runs the style interview, and persists WritingProfile updates with optional style model ingestion and consent logging.

**Independent Test**: Run `/writing start "survey on multimodal alignment"`; confirm directory + `project.json` + empty `.tex/.bib` files exist, interview responses stored, style models analyzed locally or explicitly consented before remote processing, and orchestration logs show `project_created` and `style_model_ingested` events with consent references.

### Implementation for User Story 1

- [ ] T009 [US1] Implement project slug generation + collision handling in `src/writing/project.rs`
- [ ] T010 [P] [US1] Scaffold user-layer files (`main.tex`, `sections/`, `.bib`) in `src/writing/project.rs`
- [ ] T011 [US1] Implement style interview prompt flow + WritingProfile updates in `src/writing/style.rs`
- [ ] T012 [P] [US1] Implement local style model ingestion pipeline in `src/writing/style.rs`
- [ ] T013 [US1] Detect remote style analysis needs, present consent manifest, and record approval tokens in `src/writing/style.rs`
- [ ] T014 [P] [US1] Implement `/writing/projects` list/create endpoints in `src/chat/commands/writing_projects.rs`
- [ ] T015 [US1] Implement `/writing/projects/{slug}` PATCH + Archive endpoints in `src/chat/commands/writing_projects.rs`
- [ ] T016 [US1] Connect `/writing start` chat command to orchestrate slugging, interview, style models, and manifest scaffolding in `src/chat/commands/writing_start.rs`
- [ ] T017 [US1] Emit `project_created` + `style_model_ingested` orchestration events with consent metadata in `src/orchestration/events.rs`
- [ ] T018 [US1] Update quickstart docs with project creation flow in `specs/007-writing-assistant/quickstart.md`

**Checkpoint**: User Story 1 delivers MVP writing project lifecycle + style interview.

---

## Phase 4: User Story 2 - Outline & Draft Loop (Priority: P1)

**Goal**: Users request outlines, accept/reject nodes, and generate drafts with synced `.tex` files and AI-layer metadata plus transparent outline/draft events.

**Independent Test**: From an existing project, run "generate an outline" and accept selected nodes; request "Draft intro"; verify JSON + `.tex` stay in sync, `outline_created`/`draft_generated` events are logged, `.bib` warnings appear when manual edits diverge, and outline nodes can be reverted from checkpoints.

### Implementation for User Story 2

- [ ] T019 [US2] Implement outline schema management + persistence in `src/writing/outline.rs`
- [ ] T020 [P] [US2] Build outline proposal generator referencing Paper Base summaries in `src/writing/outline.rs`
- [ ] T021 [US2] Implement `/writing/projects/{slug}/outline/proposals` + `/outline/{nodeId}/status` endpoints in `src/chat/commands/writing_outline.rs`
- [ ] T022 [US2] Persist outline acceptance checkpoints for undo in `src/writing/outline.rs`
- [ ] T023 [US2] Implement draft generator that writes section `.tex` files + AI metadata in `src/writing/drafting.rs`
- [ ] T024 [US2] Implement `/writing/projects/{slug}/drafts/{nodeId}` endpoint in `src/chat/commands/writing_draft.rs`
- [ ] T025 [US2] Emit `outline_created`/`outline_modified` events when proposals are generated or accepted in `src/orchestration/events.rs`
- [ ] T026 [US2] Add citation capture + `draft_generated` event logging in `src/writing/citations.rs`
- [ ] T027 [US2] Update `.bib` sync logic for accepted outline nodes in `src/writing/citations.rs`
- [ ] T028 [US2] Add `.bib` drift detection + user warnings when manual edits diverge from Paper Base truth in `src/writing/citations.rs`
- [ ] T029 [US2] Extend quickstart with outline + draft instructions in `specs/007-writing-assistant/quickstart.md`

**Checkpoint**: Outline/draft loop operational and testable independently.

---

## Phase 5: User Story 3 - Inline Chat Edits & Citation Injection (Priority: P2)

**Goal**: Chat commands edit sections, inject citations, show diffs, and manage undo checkpoints + `UNVERIFIED` markers with `citation_flagged` transparency.

**Independent Test**: Request "tighten section 2.1 and add Smith 2021"; verify `.tex` diff summary, AI-layer revision event, `citation_flagged` events for unresolved references, undo via `revert event <id>` works without compile, and `/writing undo` reports the restored state.

### Implementation for User Story 3

- [ ] T030 [US3] Implement inline edit command parser in `src/chat/commands/writing_edit.rs`
- [ ] T031 [P] [US3] Apply structured edits + diff snippets using `latexindent` helpers in `src/writing/drafting.rs`
- [ ] T032 [US3] Implement citation verification + `UNVERIFIED` tagging and emit `citation_flagged` events in `src/writing/citations.rs`
- [ ] T033 [US3] Persist inline-edit undo checkpoints and diff hunks in `src/writing/undo.rs`
- [ ] T034 [US3] Implement `/writing/projects/{slug}/edits` endpoint wiring parser + drafting logic in `src/chat/commands/writing_edit.rs`
- [ ] T035 [US3] Implement `/writing/projects/{slug}/undo` endpoint and `/writing undo` chat command using stored checkpoints in `src/chat/commands/writing_undo.rs`
- [ ] T036 [US3] Document inline edit workflow + undo instructions in `specs/007-writing-assistant/quickstart.md`

**Checkpoint**: Inline editing & citation injection usable without compile dependency.

---

## Phase 6: User Story 4 - Local Build & Preview Feedback (Priority: P3)

**Goal**: Run local LaTeX builds, stream logs, store artifacts, and report errors via chat while honoring local-first constraints and guarding edge cases.

**Independent Test**: Execute "compile project" for clean and broken inputs; confirm `/builds/<timestamp>/` contains PDF/logs, chat surfaces file/line guidance, `compile_attempted` events include consent status, instrumentation records durations, and invoking compile before drafts exist yields a friendly no-op response.

### Implementation for User Story 4

- [ ] T037 [US4] Implement compile executor invoking `tectonic`/`pdflatex` in `src/writing/compile.rs`
- [ ] T038 [P] [US4] Parse compiler logs -> chat-friendly messages in `src/writing/compile.rs`
- [ ] T039 [US4] Store BuildSession metadata + artifacts under `/builds/` via `src/writing/compile.rs`
- [ ] T040 [US4] Implement `/writing/projects/{slug}/builds` POST/GET endpoints in `src/chat/commands/writing_compile.rs`
- [ ] T041 [US4] Wire chat command `/writing compile` + status streaming in `src/chat/commands/writing_compile.rs`
- [ ] T042 [US4] Add compile pre-check to block builds when no drafts exist and return guidance in `src/writing/compile.rs`
- [ ] T043 [US4] Log `compile_attempted` events with consent status in `src/orchestration/events.rs`
- [ ] T044 [US4] Extend quickstart compile section with troubleshooting tips in `specs/007-writing-assistant/quickstart.md`

**Checkpoint**: PDF compilation + feedback complete; user can preview drafts locally.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements affecting multiple user stories, metrics, and compliance

- [ ] T045 [P] Review `specs/007-writing-assistant/contracts/writing-assistant.openapi.yaml` vs implementation and sync any deviations
- [ ] T046 Instrument project scaffolding timing to validate SC-001 in `src/writing/project.rs`
- [ ] T047 Instrument outline-to-`.tex` sync rate to validate SC-002 in `src/writing/outline.rs`
- [ ] T048 Instrument citation resolution metrics (auto vs UNVERIFIED) to validate SC-003 in `src/writing/citations.rs`
- [ ] T049 Instrument compile duration + log completeness to validate SC-004 in `src/writing/compile.rs`
- [ ] T050 Instrument undo latency + ensure last-20-event retention to validate SC-005 in `src/writing/undo.rs`
- [ ] T051 Harden error handling + consent manifest logging in `src/orchestration/events.rs`
- [ ] T052 [P] Add telemetry hooks for undo usage + consent decisions in `src/orchestration/events.rs`
- [ ] T053 Update documentation (`docs/writing-assistant.md`, quickstart) with final screenshots/log samples
- [ ] T054 Run end-to-end quickstart flow and note follow-ups in `specs/007-writing-assistant/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

1. Phase 1 (Setup) -> Phase 2 (Foundational)
2. Phase 2 blocks all user stories
3. User stories prioritized P1 (US1, US2) -> P2 (US3) -> P3 (US4)
4. Phase 7 (Polish) follows desired user stories completion

### User Story Dependency Graph

- US1 (Project & Interview) -> enables US2, US3, US4
- US2 (Outline & Draft) -> prerequisite for US3 (edits need drafted sections) and informs US4
- US3 (Inline Edits) -> independent of compile but benefits from US2 outputs
- US4 (Compile) -> depends on US1 scaffold + US2 drafts

### Parallel Execution Opportunities

- Setup/Foundation tasks marked [P] (T003, T005) can run concurrently.
- Within US1, style model ingestion (T012) can run parallel to file scaffolding (T010) and API wiring (T014).
- US2 outline generator (T020) can parallelize with draft generator (T023) and `.bib` drift detection (T028).
- US3 structured edits (T031) parallel to undo wiring (T033) and endpoint exposure (T034).
- US4 log parsing (T038) can run parallel to BuildSession storage (T039) and compile pre-check (T042).
- Polish instrumentation tasks (T046–T050) can run in parallel once underlying stories are complete.
- Across stories: after Phase 2, different teammates can own US1 vs US2 vs US3/US4 as capacity allows.

### Implementation Strategy

- MVP = Phases 1–3 (Setup, Foundation, User Story 1). Stop and validate `/writing start` workflow.
- Incremental delivery: add US2 (outline/draft) → US3 (inline edits & undo) → US4 (compile).
- After core stories, complete instrumentation + documentation tasks to prove success criteria before `/speckit.implement`.
