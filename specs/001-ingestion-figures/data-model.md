# Data Model – Paper Ingestion, Metadata Enrichment & Figure Extraction

**Feature**: `001-ingestion-figures`
**Date**: 2025-11-18

## Entities

### IngestionBatch
- `batch_id (UUID)` – unique per batch.
- `base_id (UUID)` – owning Paper Base.
- `source_path (String)` – folder or export path.
- `status (enum: pending/running/paused/completed/failed)`.
- `counts` – processed, ingested, skipped, failed, pending_figures.
- `progress_checkpoint (String)` – last processed file relative path.
- `started_at`, `updated_at`, `completed_at` timestamps.
- Relationships: has many `MetadataRecord`s processed during batch; has optional `FigureApproval`.

### MetadataRecord
- `paper_id (UUID)` – library entry reference.
- `doi (String?)`, `title`, `authors (Vec<String>)`, `venue`, `year`, `language`.
- `keywords (Vec<String>)`, `references (Vec<String>)`.
- `dedup_status (enum: unique/duplicate/merged)`.
- `last_updated`, `source_batch_id`.
- Constraints: DOI uniqueness within Base; dedup merges recorded with provenance.

### FigureAsset
- `figure_id (UUID)`.
- `paper_id (UUID)`.
- `caption (String)`.
- `image_path (Path)` – User Layer file.
- `extraction_status (enum: pending/success/failed/manual)`.
- `approval_batch_id (UUID)` – Consent manifest reference.
- `created_at`, `updated_at`.

### ConsentManifest
- `manifest_id (UUID)`.
- `base_id (UUID)`.
- `operation_type (enum: metadata_lookup/figure_extraction)`.
- `scope (Vec<UUID> | batch reference)`.
- `approval_text (String)`.
- `approved_at (DateTime)`.
- `user_identifier` (optional, for multi-user future).
- Stored in AI Layer and linked from orchestration events.

### AuditEntry / OrchestrationEvent
- `event_id (UUID)`.
- `event_type (enum: ingestion_started, ingestion_paused, metadata_refresh, figure_extract_start, figure_extract_complete, undo)`.
- `base_id`, `batch_id`, `timestamp`.
- `payload (JSON)` containing counts/errors.
- Provides chronological history for chat commands.

### FigureGalleryConfig (for reports)
- `config_id` per Base.
- `default_enabled (bool)`.
- `last_generated_at`.
- Relationship: references figure assets included in last report.

## Relationships & Lifecycle Notes
- `IngestionBatch` → `MetadataRecord` (one-to-many). Metadata records can be refreshed via new batches; last_updated indicates freshness.
- `FigureAsset` depends on `ConsentManifest`. Undo removes figure files and assets created in the corresponding batch.
- `ConsentManifest` ties back to `OrchestrationEvent` and ensures traceability for remote operations.

## Scale & Identity
- Expect up to ~10k `MetadataRecord`s per Base; JSON storage uses chunked files per batch to ease diffing.
- `FigureAsset` images stored as PNG/JPEG under `User/<Base>/figures/batch_id/`.
- Ingestion checkpoints stored per batch to resume after interruptions.
