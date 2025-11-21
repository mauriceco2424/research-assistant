# Implementation Plan: Learning Mode Sessions

**Branch**: `008-learning-mode` | **Date**: 2025-11-21 | **Spec**: specs/008-learning-mode/spec.md  
**Input**: Feature specification from `/specs/008-learning-mode/spec.md`

## Summary

Implement chat-first learning sessions (Quiz/Oral Exam) scoped to Base/categories/papers/concepts, driven by KnowledgeProfile gaps, with corrective feedback, logged updates, undo, and regenerable artifacts stored locally.

## Technical Context

**Language/Version**: Rust (stable, Tauri backend) + TypeScript frontend assets  
**Primary Dependencies**: Tauri, serde/serde_json, chat/orchestration modules, KnowledgeProfile storage utilities  
**Storage**: Local filesystem dual-layer (User layer + AI layer per Base); session artifacts/logs under AI layer  
**Testing**: cargo test (backend), existing integration harness; manual chat flows for learning loop  
**Target Platform**: Desktop (Tauri)  
**Project Type**: Desktop app (single repo with Rust core + frontend assets)  
**Performance Goals**: Interactive chat turns; target perceived response <3s locally for question/feedback where model allows  
**Constraints**: Local-first (no hidden network), chat-first UI, regenerable artifacts, undo for latest KnowledgeProfile update  
**Scale/Scope**: Single-user desktop; sessions cover limited scoped materials (default 5 questions, extendable)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P1 Local-First**: All artifacts/logs stored locally; external models require explicit approval per batch.  
- **P2 Consent/Approval**: Any remote model usage must present manifest/approval; default to local-only.  
- **P3/P4 Dual-Layer & Regenerability**: Store session artifacts in AI layer with pointers for regeneration.  
- **P5 Chat-First**: Entire flow stays in chat; no new complex UI views.  
- **P6 Transparency/Undo**: Log orchestration events and support undo of latest KnowledgeProfile update.  
- **P7 Academic Integrity**: Maintain real references if surfaced; no fabricated citations.  
- **P8 Learning Focus**: Feature delivers interactive learning mode with recorded sessions.  
- **P9 Versioning/Alignment**: Aligns with existing schema; no exemptions requested.  
- **P10 Extensibility**: Adds non-destructive session artifacts; no breaking migrations.

Gate result: **PASS** (no exemptions required).  
Post-design check (Phase 1): **PASS** (artifacts stored locally; chat-first preserved; undo/logging planned).

## Project Structure

### Documentation (this feature)

```text
specs/008-learning-mode/
├─ plan.md              # This file
├─ research.md          # Phase 0 output
├─ data-model.md        # Phase 1 output
├─ quickstart.md        # Phase 1 output
├─ contracts/           # Phase 1 output
└─ tasks.md             # Phase 2 output (from /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├─ acquisition/
├─ bases/
├─ bin/
├─ chat/
├─ ingestion/
├─ orchestration/
├─ profiles/
├─ reports/
├─ storage/
├─ writing/
└─ lib.rs

tests/
└─ (unit/integration harness)
```

**Structure Decision**: Use the single desktop project layout already present (`src/` Rust modules for orchestration, chat, profiles, etc.; `tests/` for unit/integration). No additional project roots needed.

## Complexity Tracking

No constitution violations; no extra complexity to justify.


