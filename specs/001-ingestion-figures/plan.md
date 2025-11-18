# Implementation Plan: Paper Ingestion, Metadata Enrichment & Figure Extraction

**Branch**: `001-ingestion-figures` | **Date**: 2025-11-18 | **Spec**: `specs/001-ingestion-figures/spec.md`
**Input**: Feature specification from `specs/001-ingestion-figures/spec.md`

**Note**: This plan is generated from `/speckit.plan` and must remain aligned with the ResearchBase constitution and master spec.

## Summary

Design and implement a resilient ingestion pipeline for ResearchBase that:

- ingests large batches of local PDFs/exports with pause/resume and progress reporting,
- normalizes and refreshes metadata (DOI, authors, keywords, dedup decisions),
- optionally extracts figures with explicit per-batch consent and stores assets locally,
- persists metadata-only records when source PDFs/figures are unavailable so HTML reports remain regenerable from the AI layer,
- exposes chat-first reprocessing/audit commands with orchestration logging, undo, and strict consent manifests for any remote enrichment.

## Technical Context

**Language/Version**: Rust 1.75 (Tauri backend) + TypeScript/HTML chat UI
**Primary Dependencies**: Tauri 1.x, Rust crates (serde, anyhow, reqwest or similar HTTP client, walkdir, tokio for async processing), front-end framework (Svelte/TypeScript) for chat interactions
**Storage**: Local filesystem per Base (User Layer for PDFs/figures/reports, AI Layer for JSON/Markdown metadata, ingestion logs, consent manifests)
**Testing**: cargo test (unit + integration), mocked remote lookups; front-end smoke tests as needed (vitest/playwright)
**Target Platform**: Desktop (Windows/macOS/Linux) via Tauri
**Project Type**: Single desktop application with internal modules (`src/chat`, `src/bases`, `src/ingestion`, `src/acquisition`, `src/reports`, `src/orchestration`)
**Performance Goals**: Ingest ≥500 PDFs per batch with minute-level progress updates; report regeneration <60s for Bases up to 1,000 papers
**Constraints**: Local-first, offline-capable core; explicit consent before any remote metadata/figure operations (chat prompts + manifest logging); when remote lookups are disabled the system must fall back to local heuristics and communicate reduced accuracy
**Scale/Scope**: Single researcher per install; per-Base libraries up to ~10k papers; ingestion batches typically 50–500 PDFs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P1 – Local-First Privacy**: All ingestion outputs (PDF copies, figures, metadata) remain on local filesystem. Remote lookups are opt-in per consent manifest.
- **P2 - Explicit AI Data Sharing, Consent, and Acquisition Approval**: Figure extraction and remote metadata lookups require per-batch approval with manifest logging and undo; metadata refresh commands must re-check consent before invoking any remote heuristics.
- **P3 – Dual-Layer File Architecture Integrity**: Metadata/ingestion records stored in AI-layer JSON/Markdown; User Layer holds assets (PDFs, figures, HTML reports).
- **P4 - Regenerable Reports and Artifacts**: Reports must be regenerable from AI-layer metadata + User Layer assets (including metadata-only placeholders) without rerunning ingestion.
- **P5 – Minimal, Chat-First Interface**: All ingestion/metadata/figure commands triggered, monitored, and audited via chat (UI panels optional mirrors only).
- **P6 – AI Orchestration Transparency and User Control**: Every ingestion/figure action logged as orchestration event, with undo for last batch.
- **P7 – Academic Integrity**: Metadata enrichment and figure usage must track provenance; no fabricated references.
- **P8 – Learning Over Text Production**: Enhanced metadata/figures should support downstream learning/reporting flows (compliance maintained).
- **P9 – Predictability & Spec Alignment**: Plan/spec align; downstream templates to be updated during tasks step.
- **P10 – Extensibility Without Breaking Bases**: New ingestion artifacts must not invalidate existing Bases; migrations avoided.

**Gate Evaluation (Pre-Design)**: PASS. Feature strictly adheres to privacy, consent, dual-layer storage, chat-first orchestration, and undo requirements. No exemptions requested.

## Project Structure

### Documentation (this feature)

```text
specs/001-ingestion-figures/
��� plan.md
��� research.md
��� data-model.md
��� quickstart.md
��� contracts/
���     ingesting.yaml (OpenAPI spec)
��� tasks.md (Phase 2 via /speckit.tasks)
```

### Source Code (repository root)

```text
src/
  chat/              # Chat commands for ingestion/metadata/figures
  bases/             # Base + metadata storage helpers
  ingestion/         # Batch ingestion engine (pause/resume, progress)
  acquisition/       # Consent + figure workflow (reuse components)
  reports/           # HTML report regeneration w/ figure galleries
  orchestration/     # Event logging, undo, manifests

tests/
  integration/       # End-to-end ingestion/figure flows
  unit/              # Batch parser, metadata dedup logic
```

**Structure Decision**: Continue single Rust crate layout with feature modules mapped above; integrate new ingestion/metadata/figure components within existing directories.

## Complexity Tracking

_No constitutional violations or exceptional complexity requested._

**Gate Evaluation (Post-Design)**: PASS – Research/data-model/contracts/quickstart reinforce privacy-first ingestion, consent tracking, and chat orchestration. No constitution deviations introduced.
