# Data Model – Categorization & Editing Workflows

**Feature**: `003-category-editing`  
**Date**: 2025-11-19

## Entities

### CategoryDefinition
- `category_id (UUID)` – stable id.
- `base_id (UUID)` – owning Base.
- `name (String)` – unique per Base.
- `slug (String)` – sanitized identifier for file paths.
- `description (String)` – short summary paragraph.
- `confidence (f32)` – 0–1 from proposal engine (optional after manual edits).
- `representative_papers (Vec<UUID>)` – up to 5 sample library entry ids.
- `pinned_papers (Vec<UUID>)` – user-curated highlights (ordered).
- `figure_gallery_enabled (bool)` – whether galleries render for this category.
- `origin (enum: proposed/manual)` – provenance.
- `created_at`, `updated_at` timestamps.
- Relationships: has many `CategoryAssignment`s; embeds latest `CategoryNarrative` reference.

### CategoryAssignment
- `assignment_id (UUID)`.
- `category_id (UUID)`.
- `paper_id (UUID)` – library entry reference.
- `source (enum: auto/manual)` – who assigned.
- `confidence (f32)` – from clustering or 1.0 for manual.
- `status (enum: active/pending_review)` – indicates backlog items needing confirmation.
- `last_reviewed_at (DateTime)` – for stale detection.
- Constraints: each `(category_id, paper_id)` unique.

### CategoryNarrative
- `narrative_id (UUID)`.
- `category_id (UUID)` (1–1).
- `summary (Markdown)` – main narrative text.
- `learning_prompts (Vec<String>)` – short bullet prompts.
- `notes (Vec<String>)` – TODOs or manual comments.
- `references (Vec<UUID>)` – papers cited in summary.
- `ai_assisted (bool)` – whether remote suggestion used.
- `last_updated_at (DateTime)`.

### CategorySnapshot
- `snapshot_id (UUID)`.
- `base_id (UUID)`.
- `taken_at (DateTime)`.
- `files (Vec<Path>)` – list of category JSON files captured.
- `reason (String)` – e.g., "rename", "merge", "narrative_edit".
- Stored under `AI/<Base>/categories/snapshots/` for undo/redo.

### CategoryEditEvent
- `event_id (UUID)`.
- `base_id (UUID)`.
- `category_ids (Vec<UUID>)` – affected categories.
- `edit_type (enum: propose_accept, rename, merge, split, move, narrative_edit, pin_toggle, undo)`.
- `details (JSON)` – before/after metadata, counts, backlog metrics.
- `timestamp (DateTime)`.
- `initiated_by (String)` – user identifier or default "local".
- Linked to `CategorySnapshot` for undo.

### CategoryStatusMetric
- `metric_id (UUID)`.
- `category_id (UUID)` or `null` for backlog aggregate.
- `paper_count (u32)`.
- `uncategorized_estimate (u32)` – for backlog summary entries.
- `staleness_days (u32)` – days since last review.
- `overload_ratio (f32)` – share of Base papers.
- `generated_at (DateTime)` – timestamp per `categories status` run.

## Relationships & Lifecycle Notes
- `CategoryDefinition` ↔ `CategoryNarrative` maintain 1–1 mapping; updates trigger report regeneration.
- `CategoryAssignment` is re-generated when merges/splits occur but retains provenance to show manual confirmations.
- `CategorySnapshot` is captured before every edit; undo consumes the most recent snapshot and emits a new `CategoryEditEvent` documenting the rollback.
- `CategoryStatusMetric` is ephemeral but persisted for history so chat can compare previous health readings.

## Scale & Identity
- Anticipate up to 50–80 active categories per Base; `CategoryAssignment` may reference up to 10k papers in aggregate.
- Snapshots are diff-friendly JSON files (~100 KB per category) so even with 100 snapshots storage stays under a few MB.
- Metrics trimmed to last 30 days per Base to avoid unbounded growth.
