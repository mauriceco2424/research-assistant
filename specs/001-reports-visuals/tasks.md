# Tasks: Reports & Visualizations

**Input**: Design documents from `/specs/001-reports-visuals/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not explicitly requested; verification occurs via manual commands described in quickstart.md and orchestration logs.

**Organization**: Tasks are grouped by user story to enable independent implementation and validation per priority ordering (P1→P3).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no blocking dependency)
- **[Story]**: Assigned user story (US1/US2/US3). Setup/Foundational/Polish omit this label.
- All descriptions include exact file paths so each task is executable without extra context.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Ensure the repo has the dependencies and scaffolding required for subsequent phases.

- [ ] T001 Update Rust dependencies (`Cargo.toml`) to include `zip`, `walkdir`, `sha2`, and `serde_with` for report manifests, hashing, bundling, and filesystem traversal.
- [ ] T002 Create `src/reports/mod.rs` exporting placeholder modules (`config_store`, `manifest`, `build_service`, `share_builder`, `consent_registry`, `visualizations`).
- [ ] T003 [P] Add TypeScript entry point `src/chat/commands/reports.ts` that registers stub handlers for `reports regenerate/configure/share` with the chat router.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that every story depends on; must complete before user story work starts.

- [ ] T004 Implement Base-level configuration persistence (`src/reports/config_store.rs`) that reads/writes JSON defaults (figures/viz toggles, excluded assets, consent TTL) inside the AI-layer directory.
- [ ] T005 [P] Define serializable data structures for `ReportBuildRequest`, `ReportManifest`, and `ShareBundleDescriptor` in `src/reports/manifest.rs`, including hashing helpers.
- [ ] T006 [P] Build consent registry (`src/reports/consent_registry.rs`) that loads/validates `ConsentManifest` files, enforces expiry, and exposes lookup APIs for report jobs.
- [ ] T007 Implement orchestration progress helper (`src/orchestration/report_progress.rs`) that streams <=5s interval updates and wraps event logging for report jobs.

**Checkpoint**: Foundational module scaffolding complete; user story phases can proceed in parallel while sharing these utilities.

---

## Phase 3: User Story 1 – Regenerate Base Reports Locally (Priority: P1) ✅ MVP

**Goal**: Allow `reports regenerate` to deterministically rebuild global & category HTML using AI-layer data, write manifests, and stream orchestration updates.

**Independent Test**: Run `cargo run -- reports regenerate --scope all --base <BASE_ID>` on a seeded Base; verify `/User/<Base>/reports/` contains new HTML + `manifest.json`, chat output lists orchestration ID, and rerunning with unchanged AI-layer produces identical hashes.

### Implementation

- [ ] T008 [US1] Implement report build queue + job lifecycle management in `src/reports/build_service.rs` (enqueue, serialization, single active job enforcement).
- [ ] T009 [P] [US1] Create deterministic HTML renderer + asset layout helpers in `src/reports/html_renderer.rs`, consuming AI-layer narratives/metrics and writing to `/User/<Base>/reports/`.
- [ ] T010 [US1] Implement manifest writer + checksum generation in `src/reports/manifest_writer.rs`, persisting `ReportManifest` per run and referencing AI-layer snapshot IDs.
- [ ] T011 [US1] Wire `reports regenerate` chat command in `src/chat/commands/reports.ts` and backend bridge `src/chat/commands/reports.rs` to invoke `build_service` with scope parsing and overwrite confirmations.
- [ ] T012 [US1] Integrate orchestration progress + completion replies in `src/chat/handlers/report_updates.rs`, ensuring chat replies include file paths, durations, and orchestration IDs.

**Checkpoint**: Base-wide regeneration works end-to-end and can be validated independently via chat command + manifest inspection.

---

## Phase 4: User Story 2 – Add Figures & Visualizations with Consent (Priority: P2)

**Goal**: Enable `reports configure` and regeneration overrides to toggle figure galleries/visualizations while enforcing consent manifests and logging approvals.

**Independent Test**: Run `cargo run -- reports configure --base <BASE_ID> --include-figures on --visualizations concept_map,timeline`; confirm consent prompt appears, approvals stored, and subsequent regeneration embeds galleries/visualizations only after consent.

### Implementation

- [ ] T013 [US2] Extend configuration command handling in `src/chat/commands/reports.ts` and backend `src/reports/config_store.rs` to persist Base defaults + override flags per consented option.
- [ ] T014 [P] [US2] Implement consent prompt + approval logging pipeline in `src/reports/consent_registry.rs`, generating prompt manifests and storing signed `ConsentManifest` records.
- [ ] T015 [US2] Build local figure extraction + gallery renderer in `src/reports/figure_gallery.rs`, storing assets under `/User/<Base>/reports/assets/figures/<category>/` and tagging manifests with consent tokens.
- [ ] T016 [US2] Implement visualization dataset selector in `src/reports/visualizations.rs` that resolves concept maps/timelines/citation graphs per toggles and records when remote summarization is required.
- [ ] T017 [US2] Update audit manifest writing (`src/reports/manifest_writer.rs`) and chat output to flag included/excluded assets plus consent references for each regeneration.

**Checkpoint**: Report generation honors stored defaults, enforces consent, and enriches HTML with approved galleries/visualizations.

---

## Phase 5: User Story 3 – Targeted Sharing & Bundling (Priority: P3)

**Goal**: Provide `reports share` to bundle requested reports/assets with provenance, overwrite confirmations, and checksums for verification.

**Independent Test**: Execute `cargo run -- reports share --base <BASE_ID> --manifest <MANIFEST_ID> --format zip --dest ./exports/base.zip`; confirm only requested files appear in the bundle, overwrite prompts fire when needed, and a `ShareBundleDescriptor` JSON references the source manifest + consent tokens.

### Implementation

- [ ] T018 [US3] Implement bundle assembly + checksum calculation in `src/reports/share_builder.rs`, respecting include/exclude flags and limiting copies to requested assets.
- [ ] T019 [P] [US3] Add provenance summary writer `src/reports/share_manifest.rs` that stores `ShareBundleDescriptor` next to bundles and links back to `ReportManifest`.
- [ ] T020 [US3] Wire `reports share` chat/backend command handling in `src/chat/commands/reports.ts` and `src/reports/share_service.rs`, including overwrite confirmations and orchestration logging.
- [ ] T021 [US3] Ensure bundle creation logs appear in chat + AI-layer audit trail by extending `src/chat/handlers/report_updates.rs` and `src/reports/manifest.rs` with share metadata.

**Checkpoint**: Bundling workflow runs independently, producing shareable archives with verifiable provenance without touching unrelated reports.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final pass for documentation, resilience, and manual validation across stories.

- [ ] T022 [P] Update `specs/001-reports-visuals/quickstart.md` with final CLI options, consent prompts, and bundle instructions.
- [ ] T023 Run end-to-end dry runs following quickstart commands, ensuring orchestration logs + manifests align; document findings in `specs/001-reports-visuals/research.md` addendum.

---

## Dependencies & Execution Order

- **Phase 1 → Phase 2**: Setup ensures dependencies/modules exist before foundational logic.
- **Phase 2 → US1/US2/US3**: Foundational config/manifest/consent/progress helpers are prerequisites for every user story.
- **Story Ordering**: Execute in priority order (US1 → US2 → US3) for MVP delivery, though US2 & US3 can run in parallel after US1 if staffing allows.
- **Polish**: Runs after desired user stories are complete.

## Parallel Execution Examples

### User Story 1
- Run T009 (HTML renderer) in parallel with T010 (manifest writer) after T008 scaffolds the build service.
- T011 (command wiring) and T012 (chat replies) can proceed concurrently once renderer + manifest writer expose interfaces.

### User Story 2
- T014 (consent prompts) and T015 (figure gallery) can proceed in parallel after config persistence (T013).
- T016 (visualization selector) may run concurrently with T017 (manifest updates) once consent hooks are available.

### User Story 3
- T018 (bundle builder) and T019 (provenance writer) can develop concurrently; integrate them via T020 command wiring afterward.
- T021 (chat/audit logging) can parallelize with T020 once share_service exposes results.

## Implementation Strategy

1. **MVP First**: Complete Phases 1–3 to unlock deterministic regeneration (US1) before investing in optional enrichments.
2. **Incremental Enhancements**: Layer US2 (consent-driven enrichments) next, validating toggles independently; deliver US3 (bundling) afterward.
3. **Parallel Staffing**: After foundational work, dedicate separate contributors to US2 and US3 while one engineer finalizes US1.
4. **Validation Rhythm**: After each phase, rerun the relevant quickstart command to ensure manifests, orchestration logs, and chat confirmations meet acceptance criteria before moving on.
