# Research – Categorization & Editing Workflows

**Feature**: `003-category-editing`  
**Date**: 2025-11-19

## Decision Log

### Decision 1 – Local Clustering Strategy
- **Decision**: Category proposals will rely on local embeddings + TF-IDF heuristics (no remote LLM by default) using `linfa-clustering` (k-means) with silhouette scoring to produce ≥5 candidate categories. Remote LLM summarization remains optional per-command consent.
- **Rationale**: Keeps proposal runs within the 2-minute SLA even for 1,000 papers, satisfies P1 by avoiding network usage, and still provides interpretable clusters backed by representative papers.
- **Alternatives Considered**:
  - *Pure LLM-based proposals*: Rejected due to privacy risk and dependence on remote latency.
  - *Rule-only heuristics (keywords only)*: Simpler but failed to capture multi-dimensional similarities in mixed-domain libraries.

### Decision 2 – Category Snapshot & Undo Mechanism
- **Decision**: Each edit (rename/merge/split/move/narrative) captures a diff-friendly snapshot (`categories/{timestamp}.json`) plus orchestration event metadata so `category undo` can restore the previous file set and re-trigger report regeneration within 60 seconds.
- **Rationale**: Aligns with P3/P4 by keeping AI-layer artifacts human-readable, enables deterministic undo without ad hoc databases, and limits storage overhead to incremental JSON snapshots.
- **Alternatives Considered**:
  - *Database transaction log*: Overkill for local-first desktop app; complicates diffing/version control.
  - *Single backup file*: Risky because concurrent edits could overwrite state without granular history.

### Decision 3 – Consent Handling for Narrative Assistance
- **Decision**: Narrative edits default to manual user text; optional AI suggestions require a refreshed consent manifest referencing operation `category_narrative_suggest` and are disabled when global offline mode is on.
- **Rationale**: Satisfies P2 by making each remote narrative request auditable and keeps offline functionality intact; also clarifies UI copy for reduced-confidence offline fallbacks.
- **Alternatives Considered**:
  - *Implicit consent reuse from ingestion metadata*: Would blur operation boundaries and weaken audit trails.
  - *Always-on AI assistance*: Violates user expectation for manual control and offline reliability.

### Decision 4 – Category Health Metrics
- **Decision**: `categories status` pulls metrics from AI-layer assignment files plus orchestration timestamps to flag stale categories (>30 days), overloaded categories (>25% of Base), and uncategorized backlog segments grouped by inferred topic.
- **Rationale**: Provides quantifiable signals for success criteria SC-004 and keeps calculations local/offline.
- **Alternatives Considered**:
  - *Real-time DB queries*: Not available in current architecture.
  - *Manual tally in chat*: Error-prone and not scalable for 10k-paper Bases.
