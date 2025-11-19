# Implementation Plan: Categorization & Editing Workflows

**Branch**: `003-category-editing` | **Date**: 2025-11-19 | **Spec**: `specs/003-category-editing/spec.md`
**Input**: Feature specification from `specs/003-category-editing/spec.md`

**Note**: This plan is generated from `/speckit.plan` and must remain aligned with the ResearchBase constitution and master spec.

## Summary

Deliver chat-first categorization workflows that:

- generate AI-proposed category structures with confidence scores after ingestion,
- let researchers rename, merge, split, and move categories/papers with full undo and orchestration logging,
- surface backlog/health metrics plus pinned highlights directly in chat,
- keep category narratives, pinned lists, and figure gallery toggles synchronized between the AI layer and regenerated HTML reports,
- preserve local-first storage, consent gating for remote assistance, and regenerability for every edit.

## Technical Context

**Language/Version**: Rust 1.75 backend (future Tauri host) with TypeScript chat shell  
**Primary Dependencies**: serde, anyhow, chrono, uuid, rayon, whatlang, plus local clustering/narrative helpers (e.g., `linfa-clustering`, `petgraph`) — no new remote SDKs  
**Storage**: Local filesystem per Base — AI layer JSON/Markdown for categories, assignments, narratives, and orchestration snapshots; User layer HTML reports + pinned galleries  
**Testing**: `cargo test` (unit + integration) with fixtures covering 50–1,000 papers; HTML snapshot comparisons for regenerated reports  
**Target Platform**: Desktop (Windows/macOS/Linux) via Tauri runtime; validated through chat/orchestration modules until UI spec lands  
**Project Type**: Single desktop application (Rust crate with `src/` modules + `tests/` suites)  
**Performance Goals**: `categories propose` completes ≤2 minutes for Bases up to 1,000 papers; report regeneration after edits ≤60 seconds; undo confirmation visible ≤10 seconds  
**Constraints**: Local-first/offline default, explicit consent manifests for remote summarization, chat-first UX, AI-layer as source of truth, undoable orchestration events  
**Scale/Scope**: Single researcher installs; Bases up to ~10k papers, ~50 active categories, backlog visibility for ≥1,000 uncategorized entries

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P1 – Local-First Privacy**: Category definitions, narratives, snapshots, and report outputs remain on the local filesystem inside the Base; no hidden network calls.  
- **P2 – Explicit AI Data Sharing & Consent**: Any remote clustering/narrative assistance requires per-command consent manifests and honors global offline switches.  
- **P3 – Dual-Layer Architecture**: AI-layer JSON/Markdown stores category structures and undo checkpoints; User layer holds regenerated HTML.  
- **P4 – Regenerability**: Every edit writes deterministic AI-layer sources so reports can be regenerated without rerunning suggestions.  
- **P5 – Minimal, Chat-First Interface**: All workflows (propose, edit, status, narrative, undo) are initiated and confirmed in chat.  
- **P6 – Orchestration Transparency & Undo**: Each edit logs orchestration events with before/after diffs plus single-step undo.  
- **P7 – Academic Integrity**: Narratives cite real papers, pinned highlights map to actual entries, AI-generated text labels confidence.  
- **P8 – Learning Over Text Production**: Updated narratives feed learning/reporting without producing unreviewed long-form prose.  
- **P9 – Spec Alignment**: Plan follows Spec 003 scope only.  
- **P10 – Extensibility Without Breaking Bases**: Category files coexist with existing metadata; no destructive migrations.

**Gate Evaluation (Pre-Design)**: PASS. Planned work satisfies privacy, consent, dual-layer, regenerability, and undo requirements with no exemptions.

## Project Structure

### Documentation (this feature)

```text
specs/003-category-editing/
��� plan.md              # This file (/speckit.plan output)
��� research.md          # Phase 0 research decisions
��� data-model.md        # Phase 1 entity definitions
��� quickstart.md        # Phase 1 operator guide
��� contracts/
���     categories.yaml  # Phase 1 OpenAPI contracts
��� tasks.md             # Produced later via /speckit.tasks
```

### Source Code (repository root)

```text
src/
  bases/             # Base metadata, AI-layer persistence, undo snapshots
  chat/              # Chat intents for categories propose/edit/status
  ingestion/         # Provides paper metadata & embeddings for clustering inputs
  reports/           # HTML regeneration logic + figure/pin rendering
  orchestration/     # Event logging, consent manifests, undo controller
  acquisition/       # Shared consent/pin helpers (optional remote ops)

tests/
  integration/       # End-to-end category propose/edit/status flows
  unit/              # Clustering heuristics, narrative diffing, undo guards
```

**Structure Decision**: Continue the single-crate layout; new categorization services span `src/chat`, `src/bases`, `src/reports`, and `src/orchestration`, with integration tests validating chat-to-storage flows.

## Complexity Tracking

_No constitutional violations or exceptional complexity requested._

**Gate Evaluation (Post-Design)**: PASS – Research/data-model/contracts/quickstart reinforce local-first categorization, consent logging, and chat orchestration with undo.
