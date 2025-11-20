# Feature Specification: Writing Assistant (LaTeX)

**Feature Branch**: `007-writing-assistant`  
**Created**: 2025-11-20  
**Status**: Draft  
**Input**: User description: "Design the Writing Assistant (LaTeX) described in `master_spec.md` §8 and roadmap Spec 07, covering chat-first style interviews, outline/draft loops, `.tex/.bib` lifecycle, inline edits, citation injection, PDF compilation, and local-only style model ingestion under the ResearchBase constitution."

## Clarifications

### Session 2025-11-20

- Q: What lifecycle states should writing projects use? → A: Draft → Active → Review → Archived

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Launch Writing Project & Style Interview (Priority: P1)

Researcher starts a writing project from chat, completes a style interview, selects or ingests style models, and receives a ready-to-edit LaTeX workspace scoped to their active Base.

**Why this priority**: Nothing else works without a correctly scoped project directory, WritingProfile context, and consent-aware style ingestion.

**Independent Test**: Trigger `/writing start "survey on multimodal alignment"` in an empty Base and verify the assistant creates `WritingProjects/<slug>/`, runs interview prompts, persists preferences, and surfaces a summary without needing downstream drafting features.

**Acceptance Scenarios**:

1. **Given** the user has no writing projects, **When** they initiate "help me write...", **Then** the assistant asks the interview, proposes a slug, and scaffolds config + empty `.tex/.bib` files under the Base path.
2. **Given** the user uploads PDFs for style modeling, **When** local analysis completes, **Then** the WritingProfile stores model metadata and any remote processing requirement is surfaced with consent logging before proceeding.

---

### User Story 2 - Outline & Draft Loop (Priority: P1)

User asks for an outline, reviews section proposals, and requests draft content that references Base artifacts and citations while keeping AI-layer outline + draft metadata in sync with `.tex` sections.

**Why this priority**: Outline acceptance governs the structure the drafts and citations depend on and is core to the assistant’s value.

**Independent Test**: With an existing project, request an outline and drafts; verify acceptance/rejection works purely in chat, `.tex` sections get generated only for accepted nodes, and outline JSON stays consistent without invoking compile or inline edit flows.

**Acceptance Scenarios**:

1. **Given** an active project with WritingProfile context, **When** the user says "generate an outline", **Then** the assistant returns a JSON outline preview, lets the user accept/modify nodes, and stores accepted outline data in AI memory with timestamps.
2. **Given** sections are accepted, **When** the user requests "Draft the intro citing last 3 ingestion highlights", **Then** the assistant creates or updates the corresponding `.tex` section with citations tied to Base entries and logs `draft_generated`.

---

### User Story 3 - Inline Chat Edits & Citation Injection (Priority: P2)

Researcher references specific section IDs or provides inline text; the assistant applies edits, injects citations, responds with diff summaries and undo tokens, and labels unverified citations per P7.

**Why this priority**: Enables iterative co-authoring without manual file editing while preserving transparency and undoability.

**Independent Test**: Request "tighten section 2.1 and add Smith 2021" on an existing draft; verify `.tex` diff summary, AI-layer revision event, citation verification, and ability to revert via logged checkpoint even without running outline or compile flows.

**Acceptance Scenarios**:

1. **Given** a section exists, **When** the user issues an edit command, **Then** the assistant shows changed lines, references files touched, emits an orchestration event with diff metadata, and stores an undo pointer.
2. **Given** a requested citation is not in the Paper Base, **When** the assistant cannot verify it, **Then** the draft contains `UNVERIFIED` markers and the chat response explains what evidence is missing.

---

### User Story 4 - Local Build & Preview Feedback (Priority: P3)

User tells the assistant to compile the project; the assistant runs local LaTeX, streams logs/errors back through chat, stores build artifacts, and reports status with actionable fixes.

**Why this priority**: Compilation validates `.tex/.bib` lifecycles and ensures researchers can review PDFs without leaving chat.

**Independent Test**: Run "compile project" with both valid and intentionally broken LaTeX; verify the assistant surfaces progress, attaches log excerpts, stores PDF outputs under the project folder, and no remote compilation occurs.

**Acceptance Scenarios**:

1. **Given** the local TeX toolchain is configured, **When** the user compiles, **Then** the assistant invokes the configured binary, streams log snippets, and places PDFs plus build metadata inside `/WritingProjects/<slug>/builds/<timestamp>/`.
2. **Given** compilation fails, **When** LaTeX reports an error, **Then** the assistant highlights the file/line, links back to the section ID, and logs a `compile_attempted` event with failure reasons for later auditing.

---

### Edge Cases

- User requests a project slug that already exists under the Base path → assistant must prompt to reuse, rename, or archive.
- Outline nodes accepted earlier are later deleted via edit commands → outline JSON and `.tex` structure must stay consistent and orphaned files flagged for cleanup.
- Local style model analysis detects content requiring GPU/remote inference → assistant pauses, presents manifest + consent reminder, and waits for approval before any upload.
- Citation insertion references a Base paper whose PDF is missing (`NEEDS_PDF`) → assistant may cite metadata but labels it pending and queues acquisition reminders.
- LaTeX compile invoked before drafts exist → assistant responds with friendly guidance instead of running an empty build and logs no-op event.
- Undo request occurs after repository was externally modified → assistant warns about divergence and offers manual conflict resolution steps.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST create, list, switch, and delete writing projects within `/User/<Base>/WritingProjects/<slug>/`, enforcing slug uniqueness per Base and storing a `project.json` manifest (name, description, owners, created/updated timestamps, status) whose lifecycle states progress Draft → Active → Review → Archived.
- **FR-002**: System MUST run a structured style interview (minimum: tone, target venue, section emphasis, citation density) whenever a new project starts or the user explicitly re-runs it, persisting results into the Base’s WritingProfile with version stamps.
- **FR-003**: System MUST support local ingestion of user-provided PDFs as style models; extracted metrics (syntax fingerprints, citation style) stay local, and any remote inference request MUST present manifest, consent token, and opt-out path before proceeding.
- **FR-004**: System MUST scaffold canonical LaTeX assets for each project (`main.tex`, section partials, `.bib`, build scripts/config`), aligning with AI-layer outline IDs so drafts can be regenerated deterministically.
- **FR-005**: System MUST capture outline proposals in structured JSON (node id, label, summary, source references, rationale, status) and allow accept/reject/modify actions entirely via chat, syncing with `.tex` files only after acceptance.
- **FR-006**: System MUST generate draft content upon user request, annotate inserted citations with Base IDs, log `draft_generated` events (project id, outline node, inputs used), and store enough metadata to replay the generation.
- **FR-007**: System MUST support inline edit commands (insert, rewrite, summarize, cite, delete) referencing section IDs or explicit text spans, apply them to `.tex`, and respond with diff snippets, file paths, and undo instructions complying with P6.
- **FR-008**: System MUST validate citations against the Paper Base before committing them; unresolved citations MUST be labeled `UNVERIFIED` in both chat summaries and `.tex`, and follow-up tasks queued in AI-layer memory.
- **FR-009**: System MUST manage `.bib` entries automatically by pulling metadata from the Base; manual edits from the user are allowed but change detection MUST warn if entries drift from Base truth.
- **FR-010**: System MUST orchestrate local LaTeX builds via a configurable command, stream log output to chat with file/line pointers, store compiled PDFs plus logs under timestamped build folders, and never invoke remote compilers.
- **FR-011**: System MUST capture and expose orchestration events (`project_created`, `style_model_ingested`, `outline_created`, `draft_generated`, `section_edited`, `citation_flagged`, `compile_attempted`) with consistent metadata for transparency and undo.
- **FR-012**: System MUST provide undo/redo affordances for outline and draft edits by checkpointing AI-layer payloads and offering "revert event <id>" chat commands with clear scope descriptions.
- **FR-013**: System MUST surface safety warnings when user requests exceed local-only constraints (e.g., external style inference) and record the user’s explicit approval/rejection per P2, blocking action until a response is received.

### Key Entities *(include if feature involves data)*

- **WritingProject**: Represents a scoped workspace (slug, title, baseId, participants, lifecycle status cycling Draft → Active → Review → Archived, timestamps, default compile toolchain, active outline id, referenced papers). Lives under `/User/<Base>/WritingProjects/<slug>/project.json`.
- **WritingProfile**: Extends Spec 05 profile data with style interview answers, preferred section ordering, tone sliders, citation patterns, and links to style models plus consent records for any remote analysis.
- **StyleModel**: Metadata describing analyzed PDFs (source file, extraction date, features captured, consent token, local analysis digest, optional remote manifest reference). Attached to WritingProfile entries used during drafting.
- **OutlineNode**: Structured AI-layer item capturing hierarchy (id, parentId, title, summary, evidence references, status, revision history, acceptance timestamps) aligned to `.tex` section files.
- **DraftSection**: Mapping between outline nodes and LaTeX files/snippets (file path, section id, last edited event id, associated citations, undo chain pointer).
- **CitationLink**: Relationship bridging `.bib` entries to Base papers (paperId, citeKey, verification status, last-checked timestamp, unresolved reason).
- **BuildSession**: Captures compile attempts (project id, toolchain config, start/end time, exit status, log file paths, produced artifact references).
- **OrchestrationEvent**: Canonical log entry (event id, type, actor, command payload, files touched, AI prompts, consent references) powering transparency, undo, and auditing.

### Assumptions

- Users have already configured at least one Base with ingested papers and profiles from Specs 01–06; this spec does not redefine onboarding or routing.
- A local LaTeX toolchain (e.g., `pdflatex` or `tectonic`) is available or the user will provide a path during initial project setup; the assistant only guides configuration.
- Git (or equivalent) is available for optional diff/undo support, but the assistant must still offer checkpoints even if git is not initialized.
- Prior specs provide access to Paper Base metadata, WritingProfile storage, and orchestration logging infrastructure; this spec extends but does not rebuild them.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Researchers can launch a new writing project (interview + scaffold) and start drafting within 5 minutes, with 100% of required files created under the Base path without manual filesystem work.
- **SC-002**: At least 90% of outline nodes accepted via chat automatically sync to corresponding `.tex` files and AI-layer metadata without structural mismatches detected during regression tests.
- **SC-003**: Citation verification flow resolves ≥95% of requested citations automatically from the Base; unresolved citations are flagged as `UNVERIFIED` within one chat turn and logged for follow-up.
- **SC-004**: Local compilation completes (success or actionable failure) within 2 minutes for standard-length papers, and 100% of compile attempts emit structured `compile_attempted` logs with links to output artifacts or errors.
- **SC-005**: Undo commands successfully revert 100% of automated edits within one chat turn, and orchestration logs retain at least the last 20 events per project for auditability.
