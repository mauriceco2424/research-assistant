# Feature Specification: Categorization & Editing Workflows

**Feature Branch**: `003-category-editing`  
**Created**: 2025-11-19  
**Status**: Draft  
**Input**: User description: "Design the Categorization & Editing feature set for ResearchBase, giving researchers a chat-first way to propose categories, revise structures, and keep HTML reports/AI-layer artifacts aligned while honoring P1–P10."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - AI-Proposed Categories (Priority: P1)

After completing ingestion/metadata, a researcher requests proposed categories for the active Base and reviews the AI’s suggested structure (names, sample papers, summaries) entirely in chat before applying it.

**Why this priority**: Without an initial, high-quality category structure, none of the downstream organization, learning, or reporting flows can deliver value.

**Independent Test**: Ingest ≥50 papers into a clean Base, run `categories propose`, review the preview in chat, and accept or reject proposals without touching any other feature.

**Acceptance Scenarios**:

1. **Given** the user runs `categories propose`, **When** the command finishes, **Then** at least five suggested categories with confidence scores, representative papers, and draft summaries are displayed in chat and persisted to AI-layer preview storage.
2. **Given** the user selects categories to accept, **When** acceptance completes, **Then** the Base’s AI-layer category definitions are updated, and HTML category/global reports are regenerated automatically.
3. **Given** the user rejects or edits a suggestion, **When** they respond via chat, **Then** the system records the decision (accept, rename, discard) and logs it as an orchestration event with timestamps and rationale.

---

### User Story 2 - Chat-Based Category Editing (Priority: P1)

Researchers can rename, merge, split, and move papers between categories solely via chat commands, with every edit captured in the AI layer and undone if the last change proves incorrect.

**Why this priority**: Categories evolve as the library grows; editing controls must be reliable and reversible to keep the Base trustworthy.

**Independent Test**: Starting from an existing Base with categories, issue chat commands to rename a category, merge two overlapping ones, split a large category, move selected papers, and then undo the last edit without touching other stories.

**Acceptance Scenarios**:

1. **Given** the user runs `category rename <old> <new>`, **When** the command executes, **Then** the AI-layer category name, notes, and references update, and affected reports regenerate with the new label.
2. **Given** two categories overlap, **When** the user runs `category merge A B`, **Then** papers from both categories combine into one, duplicates are reconciled, and a merge event with before/after counts is logged.
3. **Given** the user runs `category split <name> --by <rule>`, **When** the split finishes, **Then** the system proposes two or more child categories with their own summaries, awaiting user confirmation before committing.
4. **Given** the user issues `category undo`, **When** the most recent edit is reverted, **Then** the previous AI-layer snapshot and reports are restored within 60 seconds, and chat confirms the rollback.

---

### User Story 3 - Category Health & Backlog View (Priority: P2)

Researchers can request a status summary that highlights paper counts per category, pinned/high-priority entries, and any uncategorized backlog needing manual review.

**Why this priority**: Visibility into coverage and gaps keeps the Base useful and prevents unnoticed piles of uncategorized papers.

**Independent Test**: On a Base with at least 10 categories and 30 uncategorized papers, run `categories status` to receive backlog indicators, pinned highlights, and recommended actions without invoking other flows.

**Acceptance Scenarios**:

1. **Given** `categories status` is executed, **When** the response is rendered, **Then** each category line lists paper count, freshness timestamp, pinned papers (if any), and outstanding TODOs (notes, summaries, manual reviews).
2. **Given** there are uncategorized papers, **When** the status command runs, **Then** the chat output surfaces the backlog size, top subjects inferred, and a recommended next action (auto-cluster or manual assignment).
3. **Given** metrics show a category approaching overload (e.g., >25% of Base), **When** status runs, **Then** the response suggests splitting or rebalancing with quick links to start those commands.

---

### User Story 4 - Narrative Editing & Report Sync (Priority: P2)

Researchers can update category narratives (notes, priority flags, learning prompts) through chat, and see those edits reflected in regenerated HTML category/global reports without touching raw files.

**Why this priority**: Narrative edits power downstream reporting, coaching, and learning flows; keeping AI-layer and User-layer artifacts synchronized prevents drift.

**Independent Test**: Edit the narrative for any category, add a pinned paper, toggle figure gallery visibility, and regenerate reports—all from chat—then verify the HTML output reflects the changes without editing files manually.

**Acceptance Scenarios**:

1. **Given** the user edits a category narrative in chat, **When** they confirm the text, **Then** the AI-layer markdown updates with provenance, and the next report regeneration includes the new narrative.
2. **Given** the user pins or unpins a paper, **When** the action completes, **Then** the HTML report reorders highlighted papers accordingly and surfaces the pinned list in chat.
3. **Given** the user toggles figure gallery inclusion for a category, **When** reports regenerate, **Then** only categories with consented figures display galleries, and the change is logged.

---

### Edge Cases

- What happens when the AI cannot confidently propose enough categories (e.g., highly diverse ingest set)? System should fall back to a minimal set, flag low confidence, and prompt the user for manual seeds.
- How does system handle conflicting edits from concurrent sessions? The latest confirmed edit should detect divergence, request confirmation, or block until the user reconciles differences.
- What if report regeneration exceeds the 60-second SLA for large Bases? Chat should alert the user, keep orchestration logs, and recommend reducing batch size or running during off-hours while still completing the regeneration.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `categories propose` chat command that analyzes the active Base and produces at least five suggested categories with names, sample papers, confidence scores, and short summaries stored in AI-layer preview records.
- **FR-002**: System MUST allow the user to accept, edit, or reject each proposed category in chat, persisting accepted definitions (name, description, paper assignments) to AI-layer storage and regenerating HTML reports automatically.
- **FR-003**: System MUST support explicit rename, merge, split, and move commands for categories, ensuring each operation is orchestrated, logged, and reversible.
- **FR-004**: System MUST support per-paper re-assignment commands (e.g., move a single paper, bulk-select by filter) so researchers can tidy assignments without re-ingesting.
- **FR-005**: System MUST let users annotate narratives (notes, learning prompts, citations) and pin or unpin papers for each category through chat, with updates reflected in both AI-layer narratives and HTML reports.
- **FR-006**: System MUST emit orchestration events for every proposal, acceptance, edit, and undo, capturing timestamps, actor, affected categories, and rationale to satisfy P6 transparency requirements.
- **FR-007**: System MUST provide an undo command that restores the previous category snapshot (structure + assignments + narratives) and re-renders reports within 60 seconds for Bases up to 1,000 papers.
- **FR-008**: System MUST surface backlog insights via `categories status`, including uncategorized counts, stale categories (no edits in N days), and overloaded categories that may need splitting.
- **FR-009**: System MUST highlight pending manual tasks (e.g., uncategorized papers, unresolved summaries) in chat lists and offer quick actions to resolve them.
- **FR-010**: System MUST ensure any remote assistance used for summaries or clustering obtains explicit consent per batch (P2) and logs the consent manifest alongside the resulting artifacts; when offline, it must fall back to local heuristics and label reduced confidence.
- **FR-011**: System MUST guarantee that all category data, narratives, and edit histories remain stored locally within the Base’s AI layer, with HTML reports regenerated from those sources to preserve P3/P4 regenerability.
- **FR-012**: System MUST provide guardrails when a requested edit would orphan papers (e.g., deleting last category) by forcing the user to redistribute or confirm creation of a catch-all category before proceeding.

### Key Entities

- **CategoryDefinition**: Represents a category’s id, name, description, pinned paper references, consent flags (e.g., figures allowed), timestamps, and provenance for proposals vs. user edits.
- **CategoryAssignment**: Links library entries to categories with fields for confidence, assignment source (auto/manual), last reviewed timestamp, and pending actions (e.g., needs confirmation).
- **CategoryNarrative**: Markdown/JSON record storing learning prompts, summary paragraphs, pinned insights, and references so HTML reports can render consistent narratives.
- **CategoryEditEvent**: Append-only orchestration log entry capturing edit type (rename, merge, move, narrative change), actor, timestamp, and before/after snapshots for undo.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Starting from a Base with ≥50 papers, `categories propose` returns at least five category suggestions and a preview transcript within 2 minutes 95% of the time.
- **SC-002**: After any accepted edit (rename/merge/split/move/narrative), refreshed HTML reports reflect the change within 60 seconds for Bases up to 1,000 papers.
- **SC-003**: 90% of category edits executed via chat can be undone successfully, with confirmation surfaced in ≤10 seconds and no data loss.
- **SC-004**: During usability validation, researchers report (via survey) that category status summaries surface uncategorized backlog and suggested actions accurately in at least 4 out of 5 scenarios.

## Assumptions & Dependencies

- Existing ingestion, metadata, figure, and report modules from Specs 001–002 are available and provide the necessary paper metadata and HTML regeneration hooks.
- Any optional remote LLM assistance for clustering or narrative drafting will reuse the existing consent/manifest framework and is disabled by default unless the user opts in per operation.
- Bases larger than 1,000 papers may experience longer regeneration times; performance targets in this spec apply primarily to the small/medium Bases defined in the master roadmap.
