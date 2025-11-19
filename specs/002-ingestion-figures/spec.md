# Feature Specification: Paper Ingestion, Metadata Enrichment & Figure Extraction

**Feature Branch**: `002-ingestion-figures`  
**Created**: 2025-11-18  
**Status**: Draft  
**Input**: User description: "Design the Paper Ingestion & Figure Extraction system for ResearchBase (Spec 02)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Resilient Local Ingestion (Priority: P1)

A researcher launches an ingestion batch for a folder of PDFs or exports and can monitor, pause, resume, and recover from errors entirely through chat.

**Why this priority**: Reliable ingestion is the backbone for all subsequent workflows (metadata, figures, reporting), so it must work for both small and bulk libraries.

**Independent Test**: From a clean Base, ingest 100+ files with mixed good/bad PDFs, demonstrate progress updates, pause/resume, and resume after an app restart without restarting the batch.

**Acceptance Scenarios**:

1. **Given** the user starts an ingestion batch, **When** the batch runs, **Then** the chat timeline shows periodic progress updates (count, percent, ETA) and allows `pause`/`resume` commands without data loss.
2. **Given** a PDF is corrupt, **When** ingestion encounters it, **Then** the system skips it, records the failure reason, and surfaces remediation steps without halting the remaining files.
3. **Given** the user types "Show ingestion status", **When** a batch is active, **Then** the chat summary shows progress, pending enrichment/figure steps, and links to detailed logs.

---

### User Story 2 - Metadata Normalization & Deduplication (Priority: P1)

After or independent of ingestion, the system enriches metadata (DOI, authors, venue, keywords, language, references), detects duplicates, and lets the user confirm or override conflicts via chat.

**Why this priority**: Accurate metadata powers categorization, reporting, and eventual writing/learning features; quality must be guaranteed early.

**Independent Test**: Take a set of PDFs with mixed metadata quality, run the enrichment command, ensure duplicate detection prompts appear, and confirm AI-layer JSON reflects the user’s decisions.

**Acceptance Scenarios**:

1. **Given** a paper lacks DOI information, **When** metadata enrichment runs, **Then** the system attempts local heuristics and optional remote lookups (if enabled) and records the outcome (success, needs user input, failure).
2. **Given** two PDFs share a DOI, **When** deduplication runs, **Then** the user is prompted to merge or keep both; the final state is persisted to AI-layer metadata with provenance.
3. **Given** the user issues "Refresh metadata for Paper X", **When** the command runs, **Then** the system shows a before/after diff, allows the user to accept/reject changes, and logs the decision.

Remote metadata enrichment commands must always surface the consent manifest in chat; if the user declines approval, the command must fall back to local heuristics or mark the record for manual review without contacting remote services.

---

### User Story 3 - Consent-Driven Figure Extraction (Priority: P2)

The system offers optional figure extraction (images + captions + metadata) per batch, obtains explicit approval, stores extracted assets locally, and links them to HTML reports and AI-layer summaries.

**Why this priority**: Figure galleries enhance understanding and reporting but introduce privacy/licensing considerations; consent and provenance are critical.

**Independent Test**: After ingestion, run figure extraction for a batch, confirm approval prompts, verify extracted figures live under the User Layer, and generate a report with figure galleries toggled on/off.

**Acceptance Scenarios**:

1. **Given** figure extraction is requested, **When** the user approves "Extract figures for this batch", **Then** the system records the approval manifest, extracts figures locally, and stores figure metadata linked to the source papers.
2. **Given** extraction fails for certain papers, **When** the batch completes, **Then** the chat summary lists failures with reasons and offers retry/mark-as-manual commands.
3. **Given** a category/global report is regenerated with figure galleries enabled, **When** the report renders, **Then** only consented figures are displayed, showing captions and citations.

---

### User Story 4 - Chat-First Reprocessing & Audit Trails (Priority: P2)

Researchers can review ingestion/figure history, redo specific actions (e.g., re-run figure extraction for one paper), and undo the last batch—all within chat, with orchestration events for traceability.

**Why this priority**: Transparency and recoverability build trust, especially when the AI orchestrates complex workflows.

**Independent Test**: Execute ingestion + figure extraction, use chat commands to view history, reprocess one paper, and undo the last extraction batch while verifying files/metadata are rolled back.

**Acceptance Scenarios**:

1. **Given** the user types "Reprocess figures for Paper X", **When** the command runs, **Then** the system shows the prior consent state, re-prompts if needed, reruns extraction, and reports success/failure.
2. **Given** the user types "Show ingestion history for last 7 days", **When** the command executes, **Then** the chat displays batches with timestamps, counts, success/failure stats, and links to orchestration logs.
3. **Given** the user issues "Undo last figure extraction", **When** undo succeeds, **Then** the associated files are removed from the User Layer, metadata is updated, and the chat summarises what changed.

---

### Edge Cases

- Ingestion interrupted by app crash or system sleep: batch must resume from the last confirmed checkpoint without duplicating progress.
- Papers in multiple languages or scripts: metadata enrichment must identify language and handle right-to-left scripts without corrupting text.
- Locked or DRM PDFs: the system must flag them, avoid figure extraction, and guide the user to manual handling.
- Remote lookups disabled or consent withheld: metadata enrichment must degrade gracefully, using local heuristics only, clearly communicating reduced accuracy, and recording the decision in orchestration logs.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Provide chat-first commands to start/pause/resume/cancel ingestion batches with progress updates and resumable checkpoints.
- **FR-002**: Detect corrupt, locked, or unreadable files during ingestion, skip them, and log remediation guidance without halting the entire batch.
- **FR-003**: Normalize metadata (DOI, title, authors, venue, keywords, references, language) for each ingested paper, recording provenance and conflicts.
- **FR-004**: Offer per-paper or batch metadata refresh commands with before/after diffs so users can approve or reject updates.
- **FR-005**: Maintain metadata-only entries when PDFs/figures are missing, ensuring AI-layer artifacts remain regenerable.
- **FR-006**: Require explicit per-batch consent before figure extraction or remote metadata lookups; record manifests and approvals in orchestration logs.
- **FR-007**: Store extracted figures + captions in the User Layer and link them to AI-layer summaries so reports can include or exclude figure galleries per request.
- **FR-008**: Log every ingestion/figure action as an orchestration event (operation type, timestamps, approval text, scope) and support undo for at least the last batch per Base.
- **FR-009**: Provide chat commands to inspect ingestion/figure history with filters (date range, Base, status) and drill into specific batches or papers.
- **FR-010**: Ensure all ingestion outputs (PDF copies, metadata JSON/Markdown, figures, consent logs) remain local (P1) and sufficient to regenerate HTML reports (P4) without rerunning ingestion.

### Key Entities *(include if feature involves data)*

- **IngestionBatch**: Base ID, source path, counts (processed, skipped, failed, pending figures), status, progress checkpoints, timestamps.
- **MetadataRecord**: DOI, title, authors, venue, language, keywords, references, dedup status, last_updated, provenance.
- **FigureAsset**: Paper ID, figure ID, caption, image path, approval batch, extraction status, last_updated.
- **MetadataOnlyRecord**: Paper identifier, missing-artifact flags, provenance, and regenerable metadata fields stored when PDFs/figures are unavailable.
- **ConsentManifest**: Operation type (metadata lookup, figure extraction), scope (papers/batch), approval text, timestamp, orchestration event ID.
- **AuditEntry**: Consolidated view over orchestration events (ingestion, metadata refresh, figure extraction, undo) to power chat history commands.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Ingestion handles at least 500 PDFs per batch with progress updates every minute and <2% silent failures (all issues reported via chat/logs).
- **SC-002**: 95% of papers with valid DOIs are automatically matched and normalized in the initial enrichment pass; remaining 5% are surfaced for manual review within one chat session.
- **SC-003**: 100% of figure extraction batches that require remote data or file copies prompt for consent, log approval text, and can be undone without leftover files.
- **SC-004**: Category/global report regeneration (with optional figure galleries) completes in under 60 seconds for Bases up to 1,000 papers using only AI-layer + User-layer artifacts.
- **SC-005**: Chat history commands return results within 5 seconds for at least the most recent 50 batches, including timestamps, counts, approval status, and undo availability.
