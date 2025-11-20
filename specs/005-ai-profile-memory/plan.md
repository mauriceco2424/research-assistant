# Implementation Plan: AI Profiles & Long-Term Memory

**Branch**: `005-ai-profile-memory` | **Date**: 2025-11-19 | **Spec**: specs/005-ai-profile-memory/spec.md  
**Input**: Feature specification from `/specs/005-ai-profile-memory/spec.md`

**Note**: Executed per `/speckit.plan` workflow; downstream `/speckit.tasks` will build on the artifacts listed below.

## Summary

Deliver deterministic, auditable long-term AI profiles (User, Work, Writing, Knowledge) stored in the AI layer and surfaced via chat-first commands. The backend (Rust + Tauri orchestrator) must provide profile show/update/audit/export/delete/interview operations, knowledge readiness hooks for Learning Mode, consent-aware remote inference handling, and regeneration tools that replay orchestration events to rebuild JSON/HTML artifacts exactly. Success hinges on structured data models, orchestration logging, and local-first storage contracts that keep profiles trustworthy and regenerable.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021) for orchestrator + TypeScript 5.x front-end (Tauri shell).  
**Primary Dependencies**: `serde/serde_json/serde_yaml` for profile artifacts, `directories` + `walkdir` for Base discovery, `chrono` for timestamps, `uuid` for orchestration IDs, `sha2` for regeneration hashes, `zip` for exports, chat/orchestrator modules under `src/`.  
**Storage**: Local filesystem dual-layer layout (`/AI/<Base>/profiles/*.json`, `/User/<Base>/profiles/*.html/.zip`).  
**Testing**: `cargo test` unit suites plus targeted integration tests under `tests/integration` simulating chat commands + orchestration events; manual verification via CLI harness.  
**Target Platform**: Desktop-first ResearchBase app (Windows/macOS/Linux) running entirely on-device.  
**Project Type**: Single desktop project (Rust backend orchestrating AI flows, TypeScript UI shell).  
**Performance Goals**: `profile show <type>` chat responses under 5s for 95% calls; regeneration reproduces hashes 100% of time when logs intact; interview confirmations complete within 3 chat turns after final user input.  
**Constraints**: Chat-first UX (P5), local-first storage/no silent network (P1), explicit remote consent manifests (P2), deterministic AI/User layer parity (P3/P4), orchestration transparency + undo (P6), academic integrity for knowledge evidence (P7/P8).  
**Scale/Scope**: Each Base maintains 4 profiles, dozens of knowledge entries, and hundreds of orchestration events; multiple Bases may be open concurrently without cross-contamination of profile scope settings.

## Constitution Check (Gate 1 - pre-design)

| Principle | Compliance Plan |
|-----------|-----------------|
| **P1 Local-First** | Profiles only live under `/AI/<Base>/profiles` with optional `/User/<Base>/profiles` HTML/ZIP exports; delete/export flows never touch network. |
| **P2 Consent & Acquisition** | Every remote inference requested by profile interviews emits a prompt manifest, pauses for explicit approval, records consent metadata, and links manifest IDs to orchestration events. |
| **P3 Dual-Layer** | JSON artifacts + HTML summaries remain AI/User-layer pairs with deterministic schemas; regeneration replays orchestration logs to rebuild state. |
| **P4 Regenerability** | `profile regenerate --from-history` checks hashes after replay; missing logs block regeneration with recovery instructions. |
| **P5 Chat-First** | All interactions (show/update/interview/audit/export/delete/scope) remain chat commands or HTML attachments triggered from chat; no new panes. |
| **P6 Transparency** | Every mutation logs orchestration events (who/what/when, undo tokens, consent reference) and `profile audit` exposes them. |
| **P7 Integrity / P8 Learning** | KnowledgeProfile enforces evidence references or flags as `UNVERIFIED`, exposing readiness APIs for Learning Mode without fabricating citations. |
| **P9 Versioning** | Spec+plan maintain traceable numbering (Spec 05); migrations documented so prior Bases gain empty shells without destructive changes. |
| **P10 Extensibility** | API hooks (`profile.get_*`) defined so future features consume profiles without rework; no breaking Base formats. |

**Gate Result**: PASS - no outstanding constitutional blockers pre-Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/005-ai-profile-memory/
|-- spec.md
|-- plan.md              # This file
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/           # Phase 1 OpenAPI/GraphQL specs
|-- checklists/requirements.md
```

### Source Code (repository root)

```text
src/
|-- acquisition/
|-- bases/
|-- bin/
|-- chat/
|-- ingestion/
|-- orchestration/
|-- reports/
|-- lib.rs

tests/
|-- integration/
|-- unit/
```

**Structure Decision**: Retain the single Rust workspace with feature-specific modules under `src/chat`, `src/orchestration`, and `src/bases`; extend integration tests under `tests/integration` to simulate chat commands plus filesystem interactions.

## Phase 0 - Outline & Research

**Goals**: Resolve open design decisions, capture best practices for filesystem schemas, and document consent/regeneration strategies before modeling.

**Research Tasks**
1. **Profile Artifact Schema** - Decide JSON structure for each profile, including timestamp/evidence handling and diff-friendly ordering.
2. **Orchestration Event & Consent Logging** - Determine canonical metadata fields tying profile edits to orchestration events and consent manifests.
3. **Export & Regeneration Strategy** - Confirm hashing/ZIP approach, concurrency guards, and error handling when logs are missing or writes are concurrent.

**Execution Steps**
- Mine existing modules (`src/orchestration`, `src/bases`, `src/reports`) for reusable helpers (hashing, event logs, export utilities).
- Summarize decisions + trade-offs in `research.md` using the mandated Decision/Rationale/Alternatives format.
- Ensure each research item explicitly closes any NEEDS CLARIFICATION drawn from Technical Context or dependencies.

**Deliverable**: `specs/005-ai-profile-memory/research.md` capturing final design choices; required before entering Phase 1.

## Phase 1 - Design & Contracts

**Prerequisite**: Research doc complete.

1. **Data Modeling (`data-model.md`)**
   - Detail entities from the spec (UserProfile, WorkProfile, WritingProfile, KnowledgeProfile, ProfileChangeEvent, ConsentManifest).
   - Describe fields, validation rules (e.g., `mastery_level` enum, evidence reference formats), and state transitions (interview pending -> confirmed).
   - Map storage paths and deterministic serialization order.

2. **API / Command Contracts (`contracts/`)**
   - Produce OpenAPI-style specs (or structured Markdown contracts) for chat command handlers and internal APIs such as `profile.show`, `profile.update`, `profile.audit`, `profile.export`, `profile.delete`, `profile.regenerate`, `profile.get_work_context`, `profile.get_knowledge_summary`.
   - Include request/response schemas, error codes (missing consent, stale logs), and orchestration event side-effects.

3. **Quickstart & Operational Notes (`quickstart.md`)**
   - Outline how to run profile flows locally: seed Base, run CLI/chat commands, inspect generated JSON/HTML, test audit/regeneration.
   - Document testing strategy (unit + integration) and manual verification steps for exports/deletes.

4. **Agent Context Update**
   - Execute `.specify/scripts/powershell/update-agent-context.ps1 -AgentType codex` to append new architectural notes (profile storage paths, consent logging requirements) while respecting script markers.

5. **Constitution Re-Check (Gate 2)**
   - After drafting Phase 1 artifacts, revisit P1-P10 to confirm no design drift occurred (e.g., verify exports stay local, chat-first preserved). Document results in this plan (see section below).

## Phase 2 - Implementation Planning Handoff

**Scope**: Summarize readiness for `/speckit.tasks`.

- Verify `research.md`, `data-model.md`, `contracts/`, and `quickstart.md` exist and reference each other.
- Enumerate remaining open items (if any) that must feed into `/speckit.tasks`.
- No coding yet; stop once documents plus agent context update are complete. `/speckit.tasks` will convert this plan into executable tasks grouped by user story.

## Constitution Check (Gate 2 - post-design)

Design artifacts completed in Phase 1 maintain constitutional compliance:

- **P1**: Data model + quickstart enforce on-device storage paths; contracts omit any network endpoints outside explicit consented calls.
- **P2**: Research + contracts define manifest schema, consent statuses, and failure flows; interviews cannot persist remote data without logged approval.
- **P3/P4**: Data model captures deterministic serialization + hash tracking; regenerate contract halts on mismatched hashes.
- **P5**: All user interactions remain chat commands; quickstart documents chat workflows only.
- **P6**: ProfileChangeEvent schema logs diff summaries, undo tokens, and consent references; audit command returns these fields.
- **P7/P8**: Knowledge entries require evidence or mark `UNVERIFIED`; quickstart instructs testers to verify before Learning Mode.
- **P9/P10**: Research clarifies migrations (empty shells) and API hooks (`profile.get_*`) so future specs extend without breaking Bases.

**Gate Result**: PASS - ready for `/speckit.tasks`.

## Complexity Tracking

No constitutional violations anticipated; table remains empty unless future design phases require exceptions.
