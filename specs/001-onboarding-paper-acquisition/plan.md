# Implementation Plan: Onboarding & Paper Acquisition Workflow

**Branch**: `001-onboarding-paper-acquisition` | **Date**: 2025-11-18 | **Spec**: `specs/001-onboarding-paper-acquisition/spec.md`  
**Input**: Feature specification from `specs/001-onboarding-paper-acquisition/spec.md`

**Note**: This plan is generated from `/speckit.plan` and must remain aligned
with the ResearchBase constitution and `master_spec.md`.

## Summary

Design and implement onboarding and paper acquisition flows for ResearchBase
that:

- support multi-Base creation and selection on startup,
- provide Path A onboarding for users with existing PDFs,
- provide Path B onboarding for users without PDFs via AI-assisted discovery,
- reuse a unified, consent-driven Paper Acquisition Workflow across onboarding
  and on-demand Paper Discovery,
- respect chat-first, local-first, dual-layer architecture and explicit
  consent, logging, and undo requirements from the constitution.

## Technical Context

**Language/Version**: Rust 1.75 (Tauri backend) + TypeScript/HTML (Tauri webview UI)  
**Primary Dependencies**: Tauri 1.x, Rust crates (serde, anyhow, reqwest, directories), front-end framework (e.g., Svelte/TypeScript) for the chat UI  
**Storage**: Local filesystem for User Layer (PDFs, LaTeX, HTML reports) and AI Layer (JSON/Markdown memory), as mandated by the constitution  
**Testing**: Rust integration/unit tests (cargo test) plus front-end component tests (vitest/playwright) as needed; focus on orchestration/acquisition flows  
**Target Platform**: Desktop (Windows/macOS/Linux) via Tauri  
**Project Type**: single (single desktop application with internal modules for chat, ingestion, acquisition, and reporting)  
**Performance Goals**: Responsive chat interactions and acquisition status updates for batches up to ~100 papers; ability to process larger batches (500+) without freezing the UI  
**Constraints**: Local-first, offline-capable core; no background acquisition without explicit per-batch consent; must preserve regenerability and dual-layer structure  
**Scale/Scope**: Single desktop user per install; per-Base libraries ranging from tens to tens of thousands of papers; feature scope limited to onboarding and acquisition behaviors, not full app implementation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

This feature directly involves these principles and constraints:

- **P1 – Local-First Privacy**  
  - All acquisition must store PDFs and metadata in the local User/AI layers
    only. No cloud storage or opaque third-party sync is allowed.
- **P2 – Explicit AI Data Sharing, Consent, and Acquisition Approval**  
  - The Paper Acquisition Workflow must always present candidate papers and
    obtain explicit per-batch user approval before any DOI/URL-based metadata
    or PDF retrieval. Approvals must be logged as part of orchestration events.
- **P3 – Dual-Layer File Architecture Integrity**  
  - Onboarding and acquisition must create and maintain User and AI layers for
    each Base, with no reliance on in-memory-only state.
- **P4 – Regenerable Reports and Artifacts**  
  - All reports generated as part of onboarding must be regenerable from AI
    layer records and current library contents.
- **P5 – Minimal, Chat-First Interface**  
  - All flows must be invocable and monitorable from chat, with any UI
    pickers and lists acting as optional convenience, not the only interface.
- **P6 – AI Orchestration Transparency and User Control**  
  - Acquisition batches must be logged as orchestration events; bulk actions
    (e.g., adding many papers at once) require explicit summaries and
    confirmations; undo of at least the last batch is required.
- **P7 – Academic Integrity and Citation Handling**  
  - Acquisition and onboarding must not silently fabricate papers; candidate
    suggestions must be clearly distinguished from confirmed library entries.
- **P8 – Learning and Understanding over Mere Text Production**  
  - Out of scope for this feature directly but onboarding and acquisition
    should not block later learning features from using Base contents.
- **P9 – Predictability, Versioning, and Spec Alignment**  
  - This plan must remain aligned with the master spec and constitution;
    changes to acquisition behavior must be reflected in specs.
- **P10 – Extensibility without Breaking Existing Bases**  
  - The new workflows must not require destructive migrations; existing Bases
    must remain usable when acquisition is added.

**Gate Evaluation (Pre-Design)**:

- Local-first storage (P1): **PASS**, plan assumes filesystem-based User/AI
  layers only.
- Explicit consent for acquisition (P2): **PASS**, feature is explicitly
  scoped around per-batch approval and logging; implementation must enforce
  this.
- Dual-layer integrity (P3) and regenerability (P4): **PASS**, onboarding
  will create/maintain both layers and treat reports as regenerable views.
- Chat-first UX (P5): **PASS**, all flows are required to be chat-invocable.
- Orchestration logging and undo (P6): **PASS**, explicit orchestration
  events and undo for last batch are in spec.
- Extensibility and backward compatibility (P10): **PASS**, feature adds
  behaviors without requiring destructive migrations.

No exemptions from P1-P10 are requested for this feature.

**Gate Evaluation (Post-Design)**:

After drafting `research.md`, `data-model.md`, `contracts/openapi.yaml`, and
`quickstart.md`, the design remains consistent with:

- P1/P3/P4: All data and contracts assume local filesystem-based User/AI
  layers and regenerable reports; no cloud or opaque storage is introduced.
- P2/P6: Acquisition is explicitly modeled as user-approved batches with
  orchestration events and undo semantics; no auto-download flows exist.
- P5: All interactions are described from a chat-first perspective, with any
  UI elements treated as optional views over the same operations.
- P10: No migrations or breaking changes to existing Bases are required.

GATE: **PASS** – Planning may proceed to implementation tasks.

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
  chat/              # Chat interface and intent routing
  bases/             # Paper Base management (create/select/list)
  ingestion/         # Path A ingestion (local PDFs, exports)
  acquisition/       # Paper Acquisition Workflow (DOI/URL resolution, OA fetch)
  reports/           # HTML report generation and regeneration
  orchestration/     # Orchestration events, history, undo

tests/
  integration/       # End-to-end onboarding/acquisition flows
  unit/              # Unit tests for orchestration and acquisition logic
```

**Structure Decision**: Single desktop application with feature-focused
modules under `src/` and integration/unit tests under `tests/`, keeping
onboarding and acquisition concerns separated but coordinated through
orchestration.

## Complexity Tracking
