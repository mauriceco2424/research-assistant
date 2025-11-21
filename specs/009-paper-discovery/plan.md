# Implementation Plan: Paper Discovery Consent Workflow

**Branch**: `009-paper-discovery` | **Date**: 2025-11-21 | **Spec**: specs/009-paper-discovery/spec.md  
**Input**: Feature specification from `/specs/009-paper-discovery/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Chat-first paper discovery where AI proposes metadata-only candidates for topics, KnowledgeProfile gaps, or session follow-ups; users approve batches with explicit consent before any metadata/PDF fetch; acquisitions are logged as orchestration events with prompt manifests, provenance, deduplication via stable identifiers/title+author+year, and NEEDS_PDF marking for failures, all stored locally and regenerable.

## Technical Context

**Language/Version**: Rust (backend, Tauri shell), TypeScript/React frontend (chat UI)  
**Primary Dependencies**: Tauri, serde/reqwest (network + serialization), existing orchestration/logging layer, chat/LLM client with prompt manifests  
**Storage**: Local filesystem Base (User Layer PDFs/reports, AI Layer JSON/Markdown memory); no remote storage  
**Testing**: cargo test/integration; frontend unit/e2e (e.g., vitest/Playwright)  
**Target Platform**: Desktop (Tauri) across supported OSes  
**Project Type**: Desktop app with Rust backend + webview frontend  
**Performance Goals**: Candidate list returned in chat within 30s (per SC-001)  
**Constraints**: Local-first, zero hidden network calls, explicit consent per batch, offline-safe behavior when network unavailable  
**Scale/Scope**: Single-user desktop Bases; batches sized to typical research use (dozens of papers, not massive crawls)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- P1 Local-First Privacy: Compliant; all storage remains on local Base, no hidden network calls.  
- P2 Consent/Acquisition Approval: Compliant; per-batch approval with manifest + consent logging before any fetch.  
- P3 Dual-Layer Integrity: Compliant; metadata/AI-layer records stored in structured files, regenerable.  
- P4 Regenerable Artifacts: Compliant; AI-layer + provenance enables regeneration of reports/logs.  
- P5 Chat-First: Compliant; all flows initiated and reported via chat.  
- P6 Orchestration Transparency: Compliant; orchestration events with prompt manifests and endpoints logged.  
- P7 Academic Integrity: Compliant; uses real metadata; NEEDS_PDF flags failures.  
- P8 Learning/Evidence: Not directly impacted; no conflicts.  
- P9 Versioning/Alignment: Spec and plan aligned to constitution; no exemptions requested.  
- P10 Extensibility: Compliant; dedup + additive metadata avoids destructive migrations.

## Project Structure

### Documentation (this feature)

```text
specs/009-paper-discovery/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md            # created by /speckit.tasks (not in this run)
```

### Source Code (repository root)

```text
src/
├── models/
├── services/
├── api/          # backend orchestration + acquisition
└── ui/           # chat-first frontend bindings

tests/
├── integration/
└── unit/
```

**Structure Decision**: Use existing single Tauri project with Rust backend under `src/` and shared frontend assets; tests under `tests/` for Rust plus frontend test suites in respective packages.

## Complexity Tracking

No constitutional violations or extra-project structures introduced; table not needed.
