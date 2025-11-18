# ResearchBase – Binary, Measurable Principles (Constitution Input)

These principles are written as binary, testable constraints to guide the
project constitution and future specifications.

---

## P1 – Local-First Privacy

1. All user research artifacts (PDFs, notes, LaTeX projects, reports, AI-layer
   docs, embeddings, and caches) MUST be stored on the user's local filesystem
   under a user-selectable base directory.
2. The application MUST NOT transmit raw PDFs, full paper texts, or figures to
   any remote service except through explicitly configured model API calls.
3. The application MUST maintain a single machine-readable configuration
   listing every external endpoint (model providers, search APIs, telemetry),
   and running the app with that configuration empty MUST result in a fully
   offline experience (aside from local model calls, if any).

---

## P2 - Explicit AI Data Sharing, Consent, and Acquisition Approval

1. For each AI call that sends user content off-device, the app MUST construct
   a machine-readable "prompt manifest" describing:
   - the operation type (e.g., "ingest_paper", "generate_report"),
   - the categories of data included (metadata only, excerpts, full text),
   - the destination provider identifier.
2. Before the first use of each operation type on a Base, the UI MUST present
   a human-readable summary of that prompt manifest and MUST require explicit
   user confirmation (per Base) before proceeding.
3. A global setting MUST allow the user to disable all remote AI calls; when
   disabled, any attempt to perform such an operation MUST fail with a clear,
   user-facing message and MUST NOT fall back to hidden network usage.
4. Before initiating any paper acquisition operation that may result in PDF
   downloads (DOI/URL resolution or open-access fetching), the AI MUST present
   a list of candidate papers, the planned actions (metadata lookup, PDF fetch,
   or both), and MUST obtain explicit user approval for the selected subset in
   the current session.
5. The system MUST record that approval as part of the orchestration event for
   each acquisition batch so it is possible to verify that no paper was added
   or downloaded without a corresponding user confirmation.

---

## P3 – Dual-Layer File Architecture Integrity

1. For every Paper Base, the filesystem MUST contain:
   - a User Layer directory containing PDFs, LaTeX projects, and HTML reports,
   - a distinct AI Layer directory containing structured JSON/Markdown memory.
2. No feature MAY depend solely on in-memory state for correctness: after a
   full app restart, reloading a Base from disk MUST restore all persistent
   knowledge required for ingestion, categorization, reporting, and writing.
3. All AI-layer documents MUST be stored in a diff-friendly, human-readable
   format (JSON, YAML, or Markdown) with stable keys such that changes can be
   tracked via version control.

---

## P4 – Regenerable Reports and Artifacts

1. For every HTML report file generated (category reports, global reports,
   mini-reports, visualization pages), there MUST exist a corresponding
   AI-layer source record that identifies:
   - the report type,
   - the scope (Base, category, paper set, writing project),
   - the parameters (filters, time range, visualization options).
2. Given the AI-layer source record and the current Base contents, there MUST
   be a deterministic procedure to regenerate the report, producing either:
   - a byte-identical file, or
   - a file that differs only in explicitly allowed non-semantic fields
     (timestamps, layout-only HTML attributes, or random seeds).
3. Deleting and regenerating all reports for a Base MUST NOT change any
   AI-layer records beyond the fields listed as non-semantic in (2).

---

## P5 – Minimal, Chat-First Interface

1. The primary interaction surface MUST be a chat interface; all core actions
   (ingesting papers, creating/editing categories, generating reports, starting
   writing projects, and launching learning sessions) MUST be invocable from
   chat without requiring navigation to alternative complex views.
2. The only persistent non-modal panels in the desktop UI MUST be:
   - a left sidebar containing Bases, categories, papers, and writing projects,
   - a main chat area,
   - an optional lightweight settings panel.
3. Any new feature proposal that requires adding a permanent high-complexity
   view (dashboards, multi-pane editors, timelines) MUST be rejected or
   reworked to fit within chat interactions and/or HTML reports.

---

## P6 – AI Orchestration Transparency and User Control

1. Every non-trivial AI-triggered operation that modifies persistent data
   (ingestion, bulk recategorization, report generation, profile updates,
   writing project edits) MUST be represented by a logged "orchestration
   event" containing:
   - operation type,
   - affected Base and entities,
   - timestamp,
   - AI call identifiers (if any).
2. Before executing any bulk or destructive operation that affects more than
   a single paper, category, or writing project, the app MUST present a
   human-readable summary and require explicit user confirmation.
3. Users MUST be able to view a chronological history of orchestration events
   per Base and MUST be able to undo, or at minimum roll back, the last N
   operations that changed persistent state (where N is configurable and N>=1).

---

## P7 – Academic Integrity and Citation Handling

1. When the app produces bibliographies, reference lists, or in-text citations,
   every entry MUST either:
   - be linked to a real item in the Base's library/metadata, or
   - be explicitly marked as "unverified" in the UI and output.
2. The app MUST NOT silently fabricate citations. If the AI cannot find a
   matching reference, it MUST either:
   - request user guidance, or
   - generate a clearly marked "unverified" placeholder reference.
3. When suggesting edits to user text, AI-generated content MUST be visually
   distinguishable from the original user-authored text in the editing UI,
   and there MUST be a one-step way to accept or reject each suggestion.

---

## P8 – Learning and Understanding over Mere Text Production

1. The app MUST implement at least one interactive learning mode (e.g., Q&A,
   oral-exam style questioning, or spaced mini-reports) that uses the Base
   contents to test or deepen user understanding.
2. For each learning session, the app MUST record:
   - the Base and material covered,
   - the question/answer exchanges or exercises,
   - summary notes of key concepts reviewed.
3. Writing assistance features MUST offer an option to show "supporting
   evidence" (citations or passages from the Base) for substantive claims the
   AI proposes, and when this option is enabled, each claim without evidence
   MUST be explicitly marked as unsupported.

---

## P9 – Predictability, Versioning, and Spec Alignment

1. The project MUST maintain a versioned master spec and machine-readable
   constitution; for any released app version, there MUST be a corresponding
   spec/constitution version recorded in the repository.
2. Any change that alters or adds principles in the constitution MUST trigger:
   - a semantic version bump of the constitution,
   - a review of core templates (plans, specs, tasks) to align with the new
     principles,
   - an entry in a human-readable changelog.
3. For each principle in this file, downstream specs MUST state whether the
   feature or change being defined is:
   - fully compliant,
   - intentionally exempt (with rationale),
   - or blocked pending compliance work.

---

## P10 – Extensibility without Breaking Existing Bases

1. Adding new features or data fields MUST NOT make existing Bases unusable:
   opening an older Base in a newer version of the app MUST succeed without
   requiring destructive migrations.
2. All migrations that modify on-disk data MUST be:
   - explicitly versioned,
   - reversible or accompanied by a lossless backup,
   - logged as orchestration events for the affected Bases.
3. New optional capabilities (e.g., voice input, advanced visualizations,
   external search APIs) MUST default to a disabled state and require explicit
   user opt-in per Base or per installation.
