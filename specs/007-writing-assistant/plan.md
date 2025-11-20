# Implementation Plan: Writing Assistant (LaTeX)

**Branch**: `007-writing-assistant` | **Date**: 2025-11-20 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `/specs/007-writing-assistant/spec.md`

The Writing Assistant turns the chat surface into a local-first co-author: it scaffolds LaTeX projects under `/User/<Base>/WritingProjects/<slug>/`, runs style interviews that extend the WritingProfile, manages outline/draft loops with AI-layer metadata, performs inline edits with undo checkpoints, and orchestrates local compilation while logging every orchestration event for transparency and consent compliance. Implementation relies on the existing Tauri desktop shell (Rust backend orchestrating files + TypeScript chat UI) plus new writing-focused modules that integrate Paper Base data, AI-layer memory, and local LaTeX tooling.

## Summary

Deliver a deterministic, regenerable writing workflow where chat commands map to structured operations:
- Project lifecycle management (create/list/switch/delete with manifest + lifecycle states Draft -> Active -> Review -> Archived).
- Style interviews + WritingProfile updates with optional style model ingestion that stays local unless remote inference is consented.
- Outline + draft loops tied to AI-layer JSON while `.tex/.bib` files are generated only for accepted nodes.
- Inline edits, citation injection, diff/undo events, and compile orchestration fully surfaced in chat with logging per constitution.

## Technical Context

**Language/Version**: Rust 1.75 (backend orchestration) + TypeScript 5.4 (chat intent handlers)  
**Primary Dependencies**: Tauri runtime, tokio async runtime, serde/serde_json, chrono + uuid for event ids, Paper Base service APIs, `tectonic` CLI (preferred) with `pdflatex` fallback, `latexindent` for structured edits, citeproc/csl-json helpers for `.bib`, local consent/prompt manifest utilities  
**Storage**: Local filesystem only (User Layer for `.tex/.bib`/builds, AI Layer JSON/Markdown for outlines, consent logs, orchestration events, undo checkpoints)  
**Testing**: `cargo test` for Rust modules, `pnpm vitest` for TypeScript chat flows, golden-file tests for outline JSON and `.tex` diffs, scripted compile smoke tests for Windows/macOS/Linux  
**Target Platform**: Desktop Tauri bundle (Windows/macOS/Linux) operating offline by default with optional remote style analysis gated by consent  
**Project Type**: Desktop (single Tauri project with shared Rust/TypeScript source)  
**Performance Goals**: Project scaffolding <10s, outline/draft sync <3s (excluding LLM latency), inline edit diff + undo response <1s, compile completion (success or actionable failure) <2 minutes for 20-page projects  
**Constraints**: P1 local storage, P2 manifest + consent approval before remote inference, deterministic AI-layer payloads (P3/P4), chat-only control surface (P5), event IDs + undo (P6), citation verification (P7), style guidance referencing profiles (P8), versioned manifests/schemas (P9/P10)  
**Scale/Scope**: Single researcher per Base, up to 50 concurrent projects, outlines <=150 nodes, `.bib` <=500 entries, sequential compile queue to limit resource contention

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P1 Local-First Privacy**: All assets written under `/User/<Base>/WritingProjects/`; no remote compilation or drafting. **PASS**
- **P2 Consent**: Prompt manifests + consent tokens logged whenever optional remote style inference occurs; default path is local analysis. **PASS**
- **P3 Dual-Layer**: Outline JSON, undo checkpoints, orchestration logs live in AI-layer storage parallel to `.tex`. **PASS**
- **P4 Regenerability**: Drafts can be re-created from outline JSON + stored prompts; compile artifacts reference their inputs. **PASS**
- **P5 Chat-First**: All commands/responses flow through chat; no bespoke UI beyond HTML reports. **PASS**
- **P6 Transparency/Undo**: Event ids + diff summaries + revert commands per FR-007/FR-012. **PASS**
- **P7 Integrity**: Citation workflow enforces Base lookups; `UNVERIFIED` markers for unknown references. **PASS**
- **P8 Learning**: Style guidance draws from WritingProfile + KnowledgeProfile evidence. **PASS**
- **P9 Versioning**: WritingProject manifest + outline schema include version fields for safe future extensions. **PASS**
- **P10 Extensibility**: Module boundaries keep writing services independent so future specs (learning mode, collaborative authors) can reuse them. **PASS**

## Project Structure

### Documentation (this feature)

```text
specs/007-writing-assistant/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md (generated later by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── chat/
├── orchestration/
├── profiles/
├── storage/
├── writing/
│   ├── mod.rs
│   ├── project.rs
│   ├── style.rs
│   ├── outline.rs
│   ├── drafting.rs
│   ├── citations.rs
│   └── compile.rs
├── bin/
└── lib.rs

tests/
├── writing/
│   ├── outline_tests.rs
│   ├── drafting_diffs.rs
│   └── compile_pipeline.rs
├── integration/
└── regression/
```

**Structure Decision**: Extend the existing single Tauri project by adding a `writing` module (plus focused tests) inside the Rust backend so chat intent handlers can call into cohesive APIs without another crate. This keeps orchestration shared and honors the desktop-first constraint.

## Complexity Tracking

_No constitutional violations requiring waivers._

## Phase 0 - Outline & Research

1. **Open Questions**
   - Default LaTeX compiler + fallback strategy compatible with offline desktops.
   - How undo checkpoints persist even when git is unavailable.
   - How style model analysis extracts signals locally yet surfaces remote-consent flow when needed.
2. **Research Tasks**
   - Research local LaTeX tooling for Writing Assistant context.
   - Research deterministic undo/redo strategies for file-backed AI-layer payloads.
   - Research style model feature extraction pipelines under local-first + consent constraints.
3. **Deliverable**: `research.md` documents Decision / Rationale / Alternatives for each topic; all outstanding questions resolved before Phase 1.

## Phase 1 - Design & Contracts

1. **Data Model**: `data-model.md` defines WritingProject, WritingProfile, StyleModel, OutlineNode, DraftSection, CitationLink, BuildSession, OrchestrationEvent including fields, validation, and lifecycle transitions.
2. **Contracts**: `contracts/writing-assistant.openapi.yaml` specifies API endpoints for project lifecycle, style interviews, outline/draft management, inline edits/citations, compilation, and undo operations (consumed by chat intents / automation).
3. **Quickstart**: `quickstart.md` gives QA directions to start a project, run interview, accept outline, request drafts, perform inline edits, and compile locally.
4. **Agent Context**: `.specify/scripts/powershell/update-agent-context.ps1 -AgentType codex` records new dependencies (Tectonic CLI, latexindent) + outline schema for future agents.
5. **Post-Design Constitution Check**: Confirm that refined design artifacts still satisfy P1-P10 and document result below.

## Phase 2 - Implementation Planning Preview

- Slice implementation by user story: (1) project lifecycle + interview, (2) outline/draft loop, (3) inline edits/citations, (4) compile + logging.
- Define enabling tasks: file scaffolding utilities, AI-layer schema migrations, chat command wiring, compile executor, orchestration event extensions.
- Feed `/speckit.tasks` afterwards with concrete task lists derived from this plan + contracts.

## Post-Design Constitution Check

- **P1**: Data model + contracts keep all storage local; remote style analysis still gated by consent. **PASS**
- **P2**: Contracts expose consent tokens + prompt manifest references; research confirmed user approval flow. **PASS**
- **P3/P4**: Data model defines AI-layer snapshots + undo checkpoints; ensures regenerability. **PASS**
- **P5**: Quickstart + contracts prove everything is chat-initiated via intent handlers. **PASS**
- **P6**: OrchestrationEvent schema + undo endpoint maintain transparency. **PASS**
- **P7**: CitationLink schema + verification endpoint prevent fabricated references. **PASS**
- **P8**: Style interview + WritingProfile updates keep learning hooks intact. **PASS**
- **P9/P10**: Versioned manifests and schema extend existing Bases without destructive migrations. **PASS**
