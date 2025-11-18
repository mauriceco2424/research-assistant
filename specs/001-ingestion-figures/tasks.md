---

description: "Task list for paper ingestion, metadata enrichment & figure extraction"
---

# Tasks: Paper Ingestion, Metadata Enrichment & Figure Extraction

**Input**: Design documents from `/specs/001-ingestion-figures/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Organization**: Tasks grouped by user story for independent implementation/testing.

## Phase 1: Setup (Shared Infrastructure)

- [X] T001 Update ingestion-related config defaults in `src/bases/config.rs` (checkpoint intervals, remote lookup toggles)
- [X] T002 Wire chat command stubs (`ingest start/status/pause/resume`, `metadata refresh`, `figures extract`, `history show`) in `src/chat/mod.rs`

## Phase 2: Foundational (Blocking Prerequisites)

- [X] T003 Implement `IngestionBatchStore` persistence (JSONL) in `src/ingestion/batch_store.rs`
- [X] T004 Implement metadata record persistence helpers in `src/bases/mod.rs`
- [X] T034 Implement metadata-only entry creation when ingestion skips PDFs/figures in `src/bases/mod.rs`
- [X] T005 Implement figure asset storage utilities in `src/acquisition/figure_store.rs`
- [X] T006 Extend orchestration events & undo scaffolding for ingestion/figure batches in `src/orchestration/mod.rs`
- [X] T007 Implement consent manifest recording & validation in `src/orchestration/consent.rs`
- [X] T008 Add integration test harness bootstrap in `tests/integration/mod.rs`
- [X] T035 Add integration test for metadata-only records/regeneration in `tests/integration/metadata_only.rs`

## Phase 3: User Story 1 – Resilient Local Ingestion (Priority: P1)

**Goal**: Ingest large batches with progress, pause/resume, and recovery.
**Independent Test**: Run `ingest start` on 100+ PDFs, pause/resume mid-run, restart app, and verify ingestion resumes without duplications.

- [X] T009 [US1] Implement ingestion runner (streaming scan + checkpointing) in `src/ingestion/runner.rs`
- [X] T010 [P] [US1] Add pause/resume state transitions and persistence in `src/ingestion/runner.rs`
- [X] T011 [US1] Implement corrupt file detection & skip logging in `src/ingestion/error.rs`
- [X] T012 [US1] Implement status reporting (progress metrics) in `src/ingestion/status.rs`
- [X] T013 [US1] Wire chat handlers for `ingest start/status/pause/resume` in `src/chat/mod.rs`
- [X] T014 [US1] Add integration test `tests/integration/ingestion_progress.rs`

## Phase 4: User Story 2 - Metadata Normalization & Dedup (Priority: P1)

**Goal**: Normalize metadata, detect duplicates, and support per-paper refresh.
**Independent Test**: Run `metadata refresh` on mixed-quality papers, resolve dedup prompts, confirm AI-layer metadata updates.

- [X] T015 [US2] Implement metadata extraction/enrichment pipeline in `src/ingestion/metadata.rs`
- [X] T016 [US2] Implement dedup detection & merge logic in `src/ingestion/dedup.rs`
- [X] T017 [US2] Add chat command for metadata refresh (batch + single paper) in `src/chat/mod.rs`
- [X] T036 [US2] Prompt + record consent manifests for metadata refresh commands in `src/chat/mod.rs` and `src/orchestration/consent.rs`
- [X] T037 [US2] Extend undo logic to cover metadata lookup actions in `src/orchestration/mod.rs`
- [X] T038 [US2] Implement offline-only heuristics + chat messaging when remote lookups disabled in `src/ingestion/metadata.rs` and `src/chat/mod.rs`
- [X] T039 [US2] Add language detection + RTL-safe normalization handling in `src/ingestion/metadata.rs`
- [X] T040 [US2] Implement chat-guided dedup review flow (accept/reject/merge) in `src/chat/mod.rs` and `src/ingestion/dedup.rs`
- [X] T018 [US2] Persist metadata change diffs and provenance in `src/bases/mod.rs`
- [X] T019 [US2] Add integration test `tests/integration/metadata_refresh.rs`
- [X] T041 [US2] Integration tests for consent gating, offline fallback, and multilingual dedup in `tests/integration/metadata_consent.rs`, `metadata_offline.rs`, `metadata_multilang.rs`

## Phase 5: User Story 3 – Consent-Driven Figure Extraction (Priority: P2)

**Goal**: Optional figure extraction with consent manifests, storage, and gallery support.
**Independent Test**: Run figure extraction for a batch, approve consent, verify figures stored locally and appear in regenerated reports.

- [X] T020 [US3] Implement consent prompt + approval manifest in `src/chat/mod.rs`
- [X] T021 [US3] Implement figure extraction worker (per paper) in `src/acquisition/mod.rs`
- [X] T022 [US3] Save figure assets & metadata in `src/acquisition/figure_store.rs`
- [X] T023 [US3] Update report generation for figure galleries in `src/reports/mod.rs`
- [X] T024 [US3] Log extraction batches + failures via orchestration events in `src/orchestration/mod.rs`
- [X] T025 [US3] Add integration test `tests/integration/figure_extraction.rs`

## Phase 6: User Story 4 – Chat Reprocessing & Audit Trails (Priority: P2)

**Goal**: Provide history, reprocessing, and undo commands entirely via chat.
**Independent Test**: Show history for last 7 days, reprocess one paper's figures, undo last extraction, confirm artifacts removed.

- [X] T026 [US4] Implement `history show` command with filters in `src/chat/mod.rs`
- [X] T027 [US4] Implement per-paper reprocess command (figures/metadata) in `src/chat/mod.rs`
- [X] T042 [US4] Ensure figure reprocess commands re-check consent state and log orchestration context in `src/chat/mod.rs` and `src/orchestration/mod.rs`
- [X] T028 [US4] Implement undo ingestion/figure batch logic in `src/orchestration/mod.rs`
- [X] T029 [US4] Format chat summaries with links to artifacts in `src/chat/mod.rs`
- [X] T030 [US4] Add integration test `tests/integration/history_undo.rs`
- [X] T043 [US4] Add integration test verifying figure reprocess requires consent renewal in `tests/integration/figure_reprocess_consent.rs`

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T031 Update quickstart + documentation with ingestion/figure instructions in `specs/001-ingestion-figures/quickstart.md`
- [X] T032 Improve logging & metrics (ingestion duration, figure success rate) in `src/orchestration/mod.rs`
- [X] T033 Optimize batch processing (configurable concurrency, chunk size) in `src/ingestion/runner.rs`
- [X] T044 Instrument ingestion/report duration metrics + SLA alerts in `src/orchestration/mod.rs`
- [X] T045 Add load test covering 500+ PDFs per batch in `tests/integration/ingestion_scale.rs`
- [X] T046 Add latency test for history command with ≥50 batches in `tests/integration/history_perf.rs`
- [X] T047 Measure DOI accuracy + highlight manual review backlog in `src/chat/mod.rs` and `src/ingestion/metadata.rs`

## Dependencies & Execution Order

- Setup → Foundational → US1 → (US2, US3) → US4 → Polish.
- US1 must finish before metadata or figure workflows.
- US2 and US3 can proceed in parallel once ingestion infrastructure is stable.
- US4 depends on logs/consent data from prior stories.

## Parallel Opportunities

- T001/T002 can run concurrently.
- Foundational tasks T003–T008 touching different files are parallelizable.
- After US1 completes, teams can split: one on metadata (US2) and another on figures (US3).
- Testing tasks (T014, T019, T025, T030) can run in parallel with later implementation once prerequisites ready.

## Implementation Strategy

1. Complete Phases 1–2 to establish ingestion metadata/consent stores.
2. Deliver US1 as MVP (core ingestion reliability).
3. Layer on US2 (metadata) and US3 (figures) as separate increments.
4. Finish with US4 (history/undo) and polish tasks before `/speckit.tasks` handoff to `/speckit.implement`.
