# Feature Specification: Onboarding & Paper Acquisition Workflow

**Feature Branch**: `001-onboarding-paper-acquisition`  
**Created**: 2025-11-18  
**Status**: Draft  
**Input**: User description: "Design the Onboarding & Paper Acquisition Workflow for ResearchBase, covering multi-Base creation and selection on startup; the full two-path onboarding flow (Path A: user already has PDFs; Path B: user has no papers yet); and a unified Paper Acquisition Workflow reused by onboarding and on-demand Paper Discovery, respecting chat-first, local-first, dual User/AI layer architecture, and explicit consent rules from the constitution."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and Select Paper Bases (Priority: P1)

A researcher launches ResearchBase, creates one or more Paper Bases for their
projects, and selects which Base to work in from a simple startup flow.

**Why this priority**: All subsequent onboarding, acquisition, and discovery
flows depend on having a well-defined active Base; without this, no other
feature can be used safely.

**Independent Test**: From a clean install, a user can create at least one
Base, see it in the Base list, and select it as active, then restart the app
and confirm the previously active Base is remembered and can be changed.

**Acceptance Scenarios**:

1. **Given** the app is opened for the first time, **When** the user chooses
   "Create new Paper Base" and enters a name, **Then** a new Base is created
   with its own User Layer and AI Layer directories and is set as the active
   Base.
2. **Given** multiple Bases already exist, **When** the app starts, **Then**
   the user is shown the list of Bases and can select one as the active Base
   via chat or a simple Base picker in the sidebar.
3. **Given** a Base was active in the previous session, **When** the app
   restarts, **Then** the same Base is preselected and clearly indicated as
   active, with an option to switch to another Base.

---

### User Story 2 - Onboarding with Existing PDFs (Path A) (Priority: P1)

A researcher who already has a folder of PDFs (or a citation library export)
can quickly ingest them into a new or existing Base, get initial categories,
and review generated HTML reports, all via a chat-first flow.

**Why this priority**: Many researchers already have collections; enabling them
to get value quickly from existing PDFs is a primary value proposition.

**Independent Test**: Starting from an active Base with no papers, the user
can complete a Path A flow and end up with a populated library, initial
categories, and at least one category report and one global report.

**Acceptance Scenarios**:

1. **Given** an active Base, **When** the user says in chat "I already have
   PDFs to import" and selects a folder or supported export file, **Then** the
   system ingests the PDFs/metadata into the Base's User Layer and AI Layer
   without requiring any paper downloads.
2. **Given** ingestion completes, **When** the AI proposes initial categories
   based on the ingested papers, **Then** the user can accept, rename, merge,
   or split categories via chat and see the changes reflected in the Base
   structure.
3. **Given** categories are configured, **When** the user asks for category
   and global reports, **Then** HTML reports are generated in the User Layer
   and can be opened in a browser, with enough summary content to be useful
   even if no acquisition workflow has been used.

---

### User Story 3 - Onboarding without Existing PDFs (Path B) (Priority: P2)

A researcher with no local PDFs can describe their research area and goals in
chat; the AI proposes candidate papers, the user selects which to add, and the
system attempts to retrieve metadata and open-access PDFs in a single approved
batch per operation.

**Why this priority**: It enables new researchers or new topics to get started
without a pre-existing library, while enforcing explicit consent and
local-first constraints.

**Independent Test**: Starting from an empty Base, the user can complete a
Path B flow and end up with a small library of candidate papers (some with
PDFs, some as metadata-only) plus initial categories and reports, with every
acquisition step explicitly approved and logged.

**Acceptance Scenarios**:

1. **Given** an active but empty Base, **When** the user says "I don't have
   papers yet" and answers a short conversational interview (topic, questions,
   expertise level), **Then** the AI proposes a list of candidate papers with
   titles, authors, venues, years, and DOIs/IDs, but does not yet download
   anything.
2. **Given** a set of candidates, **When** the user selects some or all of
   them and explicitly approves acquisition for this batch, **Then** the
   system runs the Paper Acquisition Workflow, attempting DOI/URL-based
   metadata resolution and open-access PDF retrieval where legally allowed.
3. **Given** acquisition has run, **When** some PDFs could not be retrieved,
   **Then** the system creates metadata-only entries marked as `NEEDS_PDF`
   and the AI informs the user which specific papers require manual download
   and attachment.
4. **Given** a mix of full and metadata-only entries, **When** the user asks
   for categories and reports, **Then** the AI builds categories, summaries,
   and HTML reports based on all available library entries, clearly indicating
   where full texts are not yet available.

---

### User Story 4 - On-Demand Paper Discovery & Acquisition (Priority: P2)

While working in an existing Base, a researcher asks the AI to find new papers
relevant to a topic, gap, or project; the AI proposes candidates and the user
adds selected ones via the same Paper Acquisition Workflow used in onboarding.

**Why this priority**: Ongoing discovery is essential to keep a Base current;
reusing a single workflow reduces complexity and enforces consistent consent
and logging behavior.

**Independent Test**: From a Base with existing papers, the user can trigger a
discovery search, select candidates, approve acquisition, and see newly added
papers (with PDFs or `NEEDS_PDF` status) show up in the Base and reports.

**Acceptance Scenarios**:

1. **Given** an active Base with existing papers, **When** the user asks the
   AI to "find 10 recent papers on X" or "fill gaps in category Y", **Then**
   the AI proposes a list of candidate papers with metadata and DOIs/IDs but
   does not download PDFs yet.
2. **Given** the candidate list, **When** the user selects a subset and
   approves acquisition, **Then** the system runs the Paper Acquisition
   Workflow for only that selected subset and logs the operation as an
   orchestration event with the approved batch details.
3. **Given** acquisition completes, **When** the user views the Base browser
   or asks for updated reports, **Then** the new papers appear in the Base,
   with PDFs attached where available and `NEEDS_PDF` status for others,
   without altering unrelated papers or categories.

---

### User Story 5 - Consent, Logging, and Recovery for Acquisition (Priority: P3)

A researcher can review what acquisition operations the AI has performed on
their behalf, including which batches were approved, which services were
contacted, and which papers succeeded or failed, and can undo the last
acquisition batch if needed.

**Why this priority**: This supports transparency, user trust, and recovery
from mistakes in line with the constitution (P2, P6).

**Independent Test**: After one or more acquisition batches, the user can
view an acquisition history, see which batch corresponds to which chat
request, and roll back at least the last batch without leaving the Base in an
inconsistent state.

**Acceptance Scenarios**:

1. **Given** one or more previous acquisition batches, **When** the user says
   "show me my recent acquisition history", **Then** the system shows a
   chronological list of batches with timestamps, counts of successful vs
   failed acquisitions, and the consent message associated with each batch.
2. **Given** a recent batch, **When** the user requests to undo the last
   acquisition, **Then** the system removes newly added papers from that batch
   (and any derived artifacts that depend solely on them) while leaving other
   Bases and papers unchanged.
3. **Given** a batch included both successful and failed acquisitions, **When**
   the user views the history or related reports, **Then** failed items are
   clearly indicated as `NEEDS_PDF` or "not added" and are not silently
   misrepresented as fully ingested.

---

### Edge Cases

- What happens when DOI/URL resolution returns conflicting metadata for the
  same identifier? The system MUST choose a deterministic default (e.g., the
  most recent Crossref record) and log the conflict in the orchestration event
  for later review.
- How does the system handle acquisition attempts when remote AI calls or
  network access are disabled? Acquisition MUST fail fast with a clear chat
  message explaining that remote calls are disabled and that manual PDF
  management is required; no partial or hidden acquisition attempts are
  allowed.
- What happens if the user approves a very large batch (e.g., 500+ papers)?
  The system MUST process the batch in a way that remains responsive (e.g.,
  chunked processing with periodic progress updates) and MUST ensure that any
  partial failure is clearly reported and does not leave the Base in an
  inconsistent state.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support creating, listing, and selecting multiple
  Paper Bases, each with its own User Layer and AI Layer directories, and must
  persist the last active Base across restarts.
- **FR-002**: System MUST provide a chat-first onboarding flow for Path A
  (user has PDFs) that ingests PDFs or supported library exports into the
  active Base without performing any network-based acquisition.
- **FR-003**: System MUST provide a chat-first onboarding flow for Path B
  (user has no PDFs) that collects research context via conversational
  interview and uses it to propose candidate papers with metadata and
  DOIs/IDs, without downloading PDFs until explicitly approved.
- **FR-004**: System MUST implement a unified Paper Acquisition Workflow
  reused by Path B onboarding and on-demand Paper Discovery, where the AI
  proposes candidate papers, the user selects a subset and explicitly approves
  acquisition, and the app then performs DOI/URL resolution and open-access
  PDF retrieval where legally permitted.
- **FR-005**: For each acquisition batch, the system MUST create library
  entries in the active Base for all selected candidates, attaching PDFs for
  successful downloads and marking unsuccessful downloads as metadata-only
  entries with a `NEEDS_PDF` flag.
- **FR-006**: System MUST log each acquisition batch as an orchestration event
  including: operation type, active Base, list of targeted identifiers,
  whether metadata and/or PDFs were retrieved, user approval text, and
  timestamps.
- **FR-007**: System MUST provide chat commands or prompts that allow the user
  to review recent acquisition history per Base, including success/failure
  summaries and links to affected papers.
- **FR-008**: System MUST support undoing at least the last acquisition batch
  per Base, removing library entries and associated artifacts created by that
  batch without affecting other Bases or earlier batches.
- **FR-009**: System MUST generate initial categories and HTML reports (per
  category and global) after onboarding (either Path A or Path B), based on
  the current library contents, and these reports MUST be regenerable from the
  AI Layer and User Layer.
- **FR-010**: System MUST ensure all onboarding and acquisition flows are
  available via chat commands or natural-language prompts, with any auxiliary
  UI elements (e.g., Base picker, candidate lists) acting as optional, not
  mandatory, fronts for the same operations.

### Key Entities *(include if feature involves data)*

- **Paper Base**: A logical and filesystem-level container for research
  artifacts and AI memory, including a User Layer directory, an AI Layer
  directory, and configuration (e.g., last active Base, acquisition settings).
- **Paper Candidate**: A potential paper identified by the AI or import
  process, represented by metadata (title, authors, venue, year) and at least
  one stable identifier (DOI, arXiv ID, or URL) before it becomes a full
  library entry.
- **Library Entry**: A paper tracked within a Base, linked to zero or more
  PDFs in the User Layer, associated metadata in the AI Layer, and a
  `NEEDS_PDF` flag if no accessible PDF is currently attached.
- **Acquisition Batch**: A single user-approved acquisition operation
  containing a set of selected Paper Candidates, with recorded results per
  candidate (metadata success/failure, PDF success/failure) and links to the
  chat context in which approval was given.
- **Orchestration Event**: A structured record describing any non-trivial
  AI-triggered operation (including acquisition batches), used for history,
  audit, and potential undo/rollback.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new user starting from an empty workspace can create an active
  Base and complete Path A or Path B onboarding in under 15 minutes, ending
  with at least 10 papers in their library and at least one usable HTML
  report.
- **SC-002**: For Path B and on-demand Paper Discovery flows, 100% of
  acquisition operations that result in network calls are preceded by an
  explicit batch approval step visible in the chat history and logged as an
  orchestration event.
- **SC-003**: In at least 95% of acquisition attempts where a DOI/ID is valid
  and an open-access PDF is available via configured services, the system
  successfully retrieves and attaches the PDF in a single approved batch.
- **SC-004**: After any acquisition batch, users can locate and interpret the
  acquisition history for that batch (including which papers succeeded,
  failed, or are `NEEDS_PDF`) in under 2 minutes using only the chat
  interface and standard UI elements.
- **SC-005**: Undoing the last acquisition batch must complete within 30
  seconds for batches of up to 50 papers and must leave the Base in a
  consistent state with no orphaned files or AI-layer records.
