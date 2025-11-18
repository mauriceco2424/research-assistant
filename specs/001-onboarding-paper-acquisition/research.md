# Research – Onboarding & Paper Acquisition Workflow

**Feature**: `001-onboarding-paper-acquisition`  
**Spec**: `specs/001-onboarding-paper-acquisition/spec.md`  
**Date**: 2025-11-18

This document consolidates decisions for the implementation plan. All open
questions from the Technical Context have been resolved with reasonable
defaults where the master spec and constitution allow it.

---

## 1. Platform and Language Decisions

### Decision

Treat the feature design as **platform-agnostic at the spec/plan level**, but
assume a **desktop application with a local filesystem** and a **chat-centric
UI layer**. Do not lock in a specific framework (Electron, Tauri, native) in
this plan.

### Rationale

- The master spec emphasizes "desktop-first" and local filesystem storage but
  does not commit to a stack.
- The constitution requires local-first, chat-first behavior; these can be
  implemented on multiple stacks.
- Locking in a particular technology in the plan would leak implementation
  details into what should remain a technology-agnostic design.

### Alternatives Considered

- **Electron/Node**: Popular for cross-platform desktop apps, good ecosystem,
  but implies JS/TS stack.
- **Tauri/Rust**: Lightweight, security-focused, good fit for local-first apps,
  but higher barrier for some teams.
- **Native (e.g., .NET/WPF, Swift, etc.)**: Strong OS integration, but
  platform-specific and not specified by the master spec.

---

## 2. Storage and Data Layout

### Decision

Use the **dual-layer filesystem structure** mandated by the constitution:

- **User Layer**: PDFs, LaTeX projects, HTML reports, exports, extracted
  figures.
- **AI Layer**: JSON/Markdown documents capturing categories, summaries,
  acquisition batches, orchestration events, and other long-term memory.

All onboarding and acquisition workflows MUST read/write through these layers.

### Rationale

- Directly aligns with P1 (Local-First Privacy) and P3 (Dual-Layer Integrity).
- Ensures reports and knowledge are regenerable (P4).
- Keeps the implementation debuggable and version-control-friendly.

### Alternatives Considered

- Database-centric storage (SQLite or similar) for everything:
  - Rejected because it hides state, complicates regenerability, and conflicts
    with the constitution's emphasis on human-readable AI-layer artifacts.
- Mixed approach (database + files):
  - Could be valid later, but would require additional governance; out of
    scope for this feature.

---

## 3. Testing Approach

### Decision

Plan for:

- **Integration tests** for end-to-end onboarding and acquisition flows:
  - Path A onboarding (existing PDFs).
  - Path B onboarding (no PDFs, AI-suggested candidates).
  - On-demand discovery using the shared Paper Acquisition Workflow.
  - Acquisition history and undo.
- **Unit tests** for orchestration and acquisition logic:
  - Candidate list handling.
  - Consent/approval checks.
  - Result mapping (PDF vs `NEEDS_PDF`).

Framework choice is left to implementation; the plan only requires that such
tests exist and can exercise the workflows without real network calls.

### Rationale

- The critical behaviors are orchestration, consent, and correct persistence.
- Integration tests can run against mocked network/adapters to avoid external
  dependencies while preserving behavior.

### Alternatives Considered

- Rely only on manual testing:
  - Rejected; too fragile given the importance of consent/logging.
- Unit tests only:
  - Rejected; integration behavior is central to user experience.

---

## 4. Paper Acquisition Workflow Details

### Decision

Standardize a **Paper Acquisition Workflow** that:

1. Constructs a **candidate list** with metadata and stable identifiers
   (DOI/ID/URL) but no PDFs yet.
2. Presents the list in chat (and optionally UI) and requires the user to
   explicitly select papers and approve acquisition for that batch.
3. Attempts **metadata resolution** and, where legally permitted,
   **open-access PDF retrieval** for each selected candidate.
4. Creates or updates **Library Entries** in the Base with:
   - full PDF attachment when successful;
   - metadata-only entries marked `NEEDS_PDF` when PDF retrieval fails.
5. Logs the operation as an **Acquisition Batch** and **Orchestration Event**
   with explicit reference to the approval context.

### Rationale

- Aligns directly with P2 (explicit consent and acquisition approval) and
  P6 (orchestration transparency).
- Reuses the same pattern across Path B onboarding and on-demand discovery,
  reducing complexity and making behavior consistent.

### Alternatives Considered

- Auto-download on suggestion without explicit user approval:
  - Rejected as a direct violation of P2 and privacy principles.
- Separate workflows for onboarding vs discovery:
  - Rejected; unnecessary duplication and risk of inconsistent behavior.

---

## 5. Orchestration Events and Undo

### Decision

Treat each acquisition as an **atomic batch operation** with:

- A unique batch identifier.
- A list of candidate identifiers and their results (metadata-only vs PDF).
- A link to the chat message or command that included the user's approval.
- A record of which files and AI-layer records were created or modified.

Implement **undo of the last batch per Base** by:

- Removing or reverting Library Entries created by that batch.
- Removing or reverting AI-layer records created solely for those entries.
- Leaving unrelated Bases and earlier batches untouched.

### Rationale

- Supports P6 (transparency and user control) and P10 (safe extensibility).
- Batch granularity matches user mental model ("that approvals step").

### Alternatives Considered

- Per-paper undo only:
  - More granular, but harder to expose and reason about; batch-level undo
    matches user approvals.
- No undo:
  - Rejected; conflicts with P6 requirement for rollback of at least the last
    N operations.

---

## 6. Chat-First UX and Auxiliary UI

### Decision

- All onboarding and acquisition operations MUST be triggerable and
  inspectable via chat.
- Auxiliary UI elements (e.g., candidate list selection UI, Base picker) are
  allowed but must map one-to-one to chat commands/intents and must not
  introduce hidden behaviors.

### Rationale

- Aligns with P5 (Minimal, Chat-First Interface).
- Ensures all behavior is visible and scriptable via chat logs.

### Alternatives Considered

- Complex wizard-style multi-screen onboarding:
  - Rejected; adds UI complexity and conflicts with the chat-first principle.

---

## 7. Edge Case Handling Decisions

### Conflicting Metadata

- **Decision**: When multiple metadata records exist for the same DOI/ID,
  prefer a deterministic primary source (e.g., first configured provider) and
  record the conflict in the Acquisition Batch metadata.

### Network Disabled or Remote AI Calls Off

- **Decision**: If acquisition requires network access but the user has
  disabled remote calls, fail fast with a clear chat message and do not
  perform partial acquisition. Provide guidance on manual PDF management.

### Large Batches

- **Decision**: For large batches (e.g., 500+ papers), process in chunks while
  streaming progress updates to chat. Ensure that each chunk's results are
  recorded in the same Acquisition Batch record.

---

## 8. Assumptions

- The app already has or will have:
  - a basic chat interface and intent routing layer,
  - filesystem access for reading/writing User and AI layers,
  - a pluggable mechanism for contacting metadata/DOI/OA services.
- Legal and licensing constraints for OA retrieval will be handled by the
  configuration of external services and are not decided at the plan level.

---

## 9. Implementation Stack Adoption

### Decision

Adopt **Tauri** as the desktop shell, with a **Rust backend (cargo project)**
and a **TypeScript/HTML front-end** rendered in the Tauri webview.

### Rationale

- Provides a secure, low-overhead desktop container aligned with the
  privacy-first requirement: filesystem and network capabilities are granted
  explicitly.
- Rust is well-suited for orchestrating local file operations, acquisition
  workflows, and structured logging without large runtime dependencies.
- The front-end can remain a lightweight chat UI (e.g., Svelte/TypeScript),
  keeping the system chat-first while allowing optional side panels.
- Cross-platform builds (Windows/macOS/Linux) are supported out of the box.

### Alternatives Considered

- **Electron/Node**: Simplifies web development but increases memory footprint
  and exposes a larger attack surface.
- **Native-only stacks (.NET, Swift, etc.)**: Would limit cross-platform
  support or require separate implementations per OS.

