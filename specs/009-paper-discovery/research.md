# Research: Paper Discovery Consent Workflow

**Date**: 2025-11-21  
**Branch**: 009-paper-discovery  
**Spec**: specs/009-paper-discovery/spec.md

## Findings

### Decision: Deduplication keys
- **Choice**: Prefer DOI/arXiv/eprint for identity; fallback to normalized title + first author + year.
- **Rationale**: Minimizes duplicate Base entries across sources while keeping approval history intact.
- **Alternatives considered**: Title-only fuzzy matching (too noisy); always-new records with manual merge (adds user burden).

### Decision: Consent logging surface
- **Choice**: Record consent per approval batch in orchestration events with timestamp, scope, acquisition mode, and candidate IDs.
- **Rationale**: Satisfies P2/P6 transparency and enables later audits/regeneration.
- **Alternatives considered**: Implicit consent via chat message alone (insufficient auditability); per-item consent prompts (adds friction without gain).

### Decision: Offline/remote AI handling
- **Choice**: If remote AI/prompt manifests unavailable, block discovery that depends on them and report cause; allow local-only metadata filtering when available.
- **Rationale**: Aligns with P1/P2 (no hidden calls) and keeps user informed; avoids silent degradation.
- **Alternatives considered**: Silent fallback to remote defaults (violates P1/P2); partial calls without manifest (violates P6).

### Decision: Acquisition error handling
- **Choice**: For failed PDF fetches, persist metadata with NEEDS_PDF + error reason; keep provenance and consent link.
- **Rationale**: Matches master_spec acquisition workflow and maintains regenerability.
- **Alternatives considered**: Retry loops without logging (opaque); drop failed items (data loss).
