# Feature Specification: Paper Discovery Consent Workflow

**Feature Branch**: `009-paper-discovery`  
**Created**: 2025-11-21  
**Status**: Draft  
**Input**: User description: "Introduce Paper Discovery (Spec 09) aligned with master_spec.md section 10 and constitutional P1-P10. Chat-first flow where the AI proposes candidate papers (metadata only) based on Base context or user query; the user reviews and explicitly approves selected items per batch, then the app performs metadata/PDF acquisition with logged consent. Requirements: no hidden network calls; prompt manifests for any remote model use; acquisitions recorded as orchestration events with approvals; failures become metadata-only NEEDS_PDF entries; storage remains local-first and regenerable via AI-layer records. User can request discovery by topic, gap-filling from KnowledgeProfile weaknesses, or follow-ups from recent sessions. Success criteria: (1) user can request and receive a candidate list in chat; (2) per-batch approval precedes any fetch, with clear actions (metadata vs PDF); (3) approved items are added to the Base with provenance and consent logged; (4) unsuccessful fetches are marked NEEDS_PDF with reasons; (5) all artifacts and events are local and regenerable from AI-layer data with zero hidden network calls."

## Clarifications

### Session 2025-11-21

- Q: How should duplicates be detected across sources? -> A: Use stable scholarly identifiers first (DOI/arXiv/eprint), then fallback to normalized title + first author + year to avoid duplicates across sources while keeping approvals intact.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Topic Discovery with Consent (Priority: P1)

A researcher requests paper discovery for a topic or query in chat; the AI returns a candidate list (metadata only) ranked with rationales. The researcher selects items to approve, chooses metadata-only vs metadata+PDF per batch, and the system logs consent before attempting acquisition.

**Why this priority**: Delivers the core chat-first discovery and consent flow that unlocks safe acquisition for new content.

**Independent Test**: Ask for a topic, receive candidates in chat, approve a subset with chosen acquisition mode, and verify approved items are added with consent logs while unapproved items are ignored.

**Acceptance Scenarios**:

1. **Given** the user requests topic discovery, **When** the AI proposes candidates with metadata-only details, **Then** the user can approve selected items in one batch and acquisitions run only after consent is logged.
2. **Given** the user approves a batch, **When** metadata retrieval succeeds but PDF retrieval fails, **Then** the system stores metadata, marks entries as NEEDS_PDF with reasons, and reports outcomes in chat and orchestration logs.

---

### User Story 2 - KnowledgeProfile Gap Filling (Priority: P2)

A user asks the system to propose papers that address weaknesses or gaps identified in the Base's KnowledgeProfile; the AI surfaces candidates aligned to those gaps, and the user approves acquisitions with logged consent.

**Why this priority**: Leverages existing Base context to strengthen knowledge coverage while preserving approval and provenance rules.

**Independent Test**: Trigger gap-based discovery, receive candidates tied to specific gaps, approve items, and verify added entries reference the gap rationale and retain consent + provenance logs.

**Acceptance Scenarios**:

1. **Given** KnowledgeProfile gaps exist, **When** the user requests gap-filling discovery, **Then** candidate papers are labeled with the gap they address and can be approved with logged consent before acquisition.

---

### User Story 3 - Session Follow-ups (Priority: P3)

After a recent study session, the user requests follow-up papers; the AI proposes candidates referencing the prior session context, the user approves a batch, and acquisition results (success or NEEDS_PDF) are logged and shown in chat.

**Why this priority**: Extends discovery to session-driven exploration, reusing the consented acquisition flow.

**Independent Test**: From a recent session, request follow-ups, approve items, and confirm storage, NEEDS_PDF handling, and orchestration logs reflect the session context.

**Acceptance Scenarios**:

1. **Given** a recent session context, **When** the user requests follow-up discovery, **Then** candidates cite the session and only approved items proceed to acquisition with consent recorded.

---

### Edge Cases

- No network connectivity during acquisition: inform the user in chat, log the failure, and leave approved items as metadata-only marked NEEDS_PDF with reasons.
- Remote AI or prompt manifest generation unavailable: block discovery/acquisition that depends on it, surface the reason in chat, and avoid hidden calls.
- Duplicate or already-present papers in the Base: flag duplicates in the candidate list and avoid creating duplicate entries; log the decision.
- User approves zero items in a batch: record the decision and do not attempt any network fetches.
- Partial acquisition success within a batch: store successes with provenance, mark remaining items as NEEDS_PDF with error context, and present a summary in chat.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST support chat-first discovery requests for topics, KnowledgeProfile gaps, or recent session follow-ups, returning candidate lists as metadata-only suggestions.
- **FR-002**: Candidate lists MUST include key metadata (title, authors, venue/year if available, source/link, rationale tied to topic/gap/session) and clearly state that no acquisition occurs before approval.
- **FR-003**: The user MUST be able to approve or reject candidates per batch, choosing metadata-only acquisition or metadata+PDF attempts, with explicit consent captured before any fetch.
- **FR-004**: All acquisition attempts MUST be preceded by logged user consent (timestamp, request context, selection, acquisition mode) and blocked if consent is missing or ambiguous.
- **FR-005**: The system MUST log orchestration events for each acquisition attempt, including prompt manifests for any remote AI calls, network endpoints contacted, outcomes, and reasons for failures.
- **FR-006**: Successful acquisitions MUST store metadata and retrieved PDFs locally within the Base and record AI-layer entries so artifacts are regenerable; no hidden network calls are allowed.
- **FR-007**: Failed or partial acquisitions MUST create metadata-only entries flagged NEEDS_PDF with user-visible error reasons and keep provenance of attempted sources.
- **FR-008**: The user MUST be able to review outcomes in chat, including which items succeeded, which are marked NEEDS_PDF, and which were skipped or already present.
- **FR-009**: Stored entries MUST retain provenance (discovery source, approval record, acquisition channel) to support audits and regenerability in line with constitutional P3/P6.
- **FR-010**: The system MUST prevent duplicate additions by matching existing papers via stable identifiers (DOI, arXiv/eprint) and, when absent, via normalized title + first author + year; log deduplication decisions without losing approval history.

### Key Entities *(include if feature involves data)*

- **CandidatePaper**: Metadata-only suggestion with title, contributors, venue/year, source link, rationale (topic/gap/session), and duplicate indicators.
- **ApprovalBatch**: User decision set capturing selected CandidatePapers, chosen acquisition mode (metadata-only vs metadata+PDF), timestamp, and consent record.
- **AcquisitionEvent**: Logged attempt per approved paper with endpoints contacted, prompt manifest reference (if AI-assisted), outcome (success, NEEDS_PDF, skipped), and error reason when applicable.
- **StoredPaperRecord**: Persistent entry in the Base linking metadata, PDF (if obtained), provenance (source suggestion, approval batch), and local storage paths for regenerability.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users requesting topic/gap/session discovery receive a candidate list in chat within 30 seconds in 9 of 10 test runs.
- **SC-002**: 100% of acquisition attempts in testing are preceded by an explicit logged approval record that specifies batch scope and acquisition mode.
- **SC-003**: At least 95% of approved items in testing are persisted with provenance and consent details visible from the Base’s orchestration/event history.
- **SC-004**: 100% of failed or partial acquisitions in testing result in metadata entries flagged NEEDS_PDF with a clear reason shown in chat and recorded in the log.
- **SC-005**: All remote AI calls and network fetches exercised during testing produce accessible prompt manifests/endpoints in the orchestration log, with zero hidden or unlabeled network traffic.


