# Research – Paper Ingestion, Metadata Enrichment & Figure Extraction

**Feature**: `002-ingestion-figures`
**Spec**: `specs/002-ingestion-figures/spec.md`
**Date**: 2025-11-18

## Decisions & Rationale

### 1. Ingestion Architecture
- **Decision**: Streaming batch processor with resumable checkpoints stored in AI Layer per Base; uses Rust async tasks plus `walkdir` for filesystem scanning.
- **Rationale**: Keeps memory usage low, allows pause/resume, and aligns with local-first constraints.
- **Alternatives Considered**: Loading entire batch catalog in memory (too heavy); delegating ingestion to external CLI (breaks chat-first UX).

### 2. Metadata Enrichment Strategy
- **Decision**: Tiered enrichment (local heuristics → optional remote DOI services if consent/grants). Dedup decisions stored alongside metadata with merge history.
- **Rationale**: Preserves privacy by default, while allowing richer metadata when users opt in; dedup history ensures deterministic behavior.
- **Alternatives Considered**: Always calling remote services (violates P1/P2); ignoring dedup/resolution (would break reports and writing workflows).

### 3. Figure Extraction Workflow
- **Decision**: Optional post-ingestion step that requires explicit approval per batch; extracted figures saved to User Layer/figures, metadata recorded in AI Layer.
- **Rationale**: Fulfills consent logging (P2 & P6) and ensures figures stay local.
- **Alternatives Considered**: Auto-extract all figures (privacy/licensing risk); deferring entirely to future spec (blocks figure galleries).

### 4. Orchestration & Undo
- **Decision**: Every ingestion/figure action emits JSONL orchestration events with batch IDs, approval text, counts, and file references; undo works by referencing these logs.
- **Rationale**: Satisfies P6 (transparency) and P10 (safe evolution).
- **Alternatives Considered**: Storing only summary logs (insufficient for undo); using an external database (conflicts with dual-layer requirement).

### 5. User Experience & Chat Commands
- **Decision**: Provide discrete chat commands for `ingest start`, `ingest status`, `ingest pause/resume`, `metadata refresh`, `figures extract`, `ingestion history`, `undo last extraction`.
- **Rationale**: Keeps UX consistent with chat-first philosophy and simplifies testing.
- **Alternatives Considered**: Dedicated GUI wizard (extra UI complexity), or hidden background jobs (violates transparency).

### 6. Scalability Targets
- **Decision**: Optimize for batches of ~500 PDFs per run; store progress checkpoints every 25 files; use concurrency for metadata parsing but single-writer persistence.
- **Rationale**: Matches success criteria and local disk constraints.
- **Alternatives Considered**: Fully parallel ingestion (risk of high IO contention); sequential only (too slow).

## Unresolved/Deferred Items
- External metadata provider list & credentials will be specified later when integrating real services; defaults assume local-only mode with optional remote stub.
- Advanced figure OCR/text extraction is out of scope for this spec and may be handled in a later enhancement.
