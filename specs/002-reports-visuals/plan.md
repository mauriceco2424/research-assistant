# Implementation Plan: Reports & Visualizations

**Branch**: `002-reports-visuals` | **Date**: 2025-11-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Chat commands (`reports regenerate`, `reports configure`, `reports share`) transform AI-layer narratives, metrics, and visualization datasets into deterministic HTML bundles stored under `/User/<Base>/reports/`. Consent manifests gate figure/visualization enrichment, manifests capture provenance for regenerability, and orchestration events keep users informed of progress, overwrites, and bundle outputs. The `ReportManifest` defined in this plan is the same “audit manifest” referenced throughout the specification; no parallel artifact exists.

## Technical Context

**Language/Version**: Rust 1.75 backend (Tauri shell) + TypeScript/HTML helpers  
**Primary Dependencies**: Tauri runtime, serde/serde_json, local filesystem APIs, orchestration event bus, chat command router  
**Storage**: Local filesystem per Base (reports, manifests, assets) plus AI-layer JSON/Markdown stores  
**Testing**: `cargo test` (Rust unit/integration) + TypeScript chat-command harness  
**Target Platform**: Desktop (Windows/macOS/Linux)  
**Project Type**: Single desktop project (Rust core modules + TypeScript bindings)  
**Performance Goals**: Regenerate ≤1,000-paper Bases in ≤60s with chat progress updates every ≤5s  
**Constraints**: Local-first operations, explicit consent before network use, overwrite confirmations, deterministic manifests, offline-friendly defaults  
**Scale/Scope**: Bases up to ~1,000 papers, dozens of categories, multiple figure galleries/visualization datasets

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P1 Local-First** – Report builds never leave the Base directory; remote calls only occur after opt-in manifests -> PASS.
- **P2 Consent** – Figure/viz toggles default off and require prompt manifests + stored approvals -> PASS.
- **P3 Dual-Layer** – Reports draw exclusively from AI-layer snapshots and record references, preventing drift -> PASS.
- **P4 Regenerability** – Audit manifests + stored configs guarantee deterministic rebuilds and verifiable bundles -> PASS.
- **P5 Chat-First** – All report actions originate in chat with textual progress + confirmations; no new UI panes -> PASS.
- **P6 Transparency** – Orchestration events capture start/end, asset counts, and overwrite/share confirmations -> PASS.
- **P7 Academic Integrity** – Reports cite sources, distinguish AI narratives, and document provenance for sharing -> PASS.
- **P8 Learning Alignment** – Reports feed backlog/health metrics needed for learning prompts without bypassing learning mode -> PASS.
- **P9 Versioning/Governance** – No schema or Base migrations required; manifests are diff-friendly -> PASS.
- **P10 Extensibility** – Uses existing routers/modules, so future Bases remain compatible -> PASS.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── acquisition/
├── bases/
├── chat/
├── ingestion/
├── orchestration/
├── reports/
└── lib.rs

tests/
├── integration/
└── unit/
```

**Structure Decision**: Stay with the single Tauri project. Reporting orchestrators live in `src/reports/`, orchestration hooks in `src/orchestration/`, and chat command wiring in `src/chat/`. Consent + manifest utilities sit alongside report services, with unit coverage in `tests/unit` and chat/FS regression tests in `tests/integration`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| *(none)* | | |
