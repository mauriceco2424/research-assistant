# Data Model – Reports & Visualizations

## ReportBuildRequest
- **Purpose**: Represents a chat-issued command to regenerate or share reports.
- **Fields**:
  - `id` (UUID, required) – unique per job.
  - `base_id` (UUID, required) – owning Base.
  - `command` (enum: regenerate/configure/share).
  - `scope` (enum: all | global_only | category_ids[] | custom include set).
  - `config_snapshot_id` (UUID) – references saved Base defaults applied to this run.
  - `overrides` (map<string, value>) – ad-hoc flags provided in command.
  - `requested_assets` (set<string>) – figures, viz types explicitly requested.
  - `created_at`, `started_at`, `completed_at` timestamps.
  - `status` (enum: queued | running | succeeded | failed | cancelled).
- **Relationships**: Links to `ReportManifest`(1:1) when generation succeeds; references `ConsentManifest` tokens used.
- **Validation Rules**:
  - Cannot transition to `running` without satisfied consent requirements.
  - Only one `running` request per Base scope at a time (queue/confirm to avoid overlap).
- **State Transitions**: `queued → running → succeeded/failed`; optional `running → cancelled`.

## ReportManifest
- **Purpose**: Deterministic description of generated HTML output.
- **Fields**:
  - `manifest_id` (UUID) + `version` (int).
  - `base_id`, `build_request_id` references.
  - `ai_layer_snapshots` (list of snapshot IDs per category/global).
  - `metrics_revision_id`, `visualization_dataset_ids`.
  - `config_signature` (hash of Base defaults + overrides).
  - `consent_tokens` (array of ConsentManifest IDs included).
  - `outputs` (list of objects: `path`, `hash`, `type`, `scope`).
  - `duration_ms`, `orchestration_id`.
- **Relationships**: 1:many with `ShareBundleDescriptor` (each bundle cites a manifest).
- **Validation Rules**:
  - `outputs` must cover every scope requested or mark as `unchanged` with reason.
  - Hashes required for integrity checks.

## ConsentManifest
- **Purpose**: Records approvals for figure extraction or remote summarization.
- **Fields**:
  - `consent_id` (UUID), `base_id`.
  - `operation` (figure_gallery_render, visualization_remote_layout, etc.).
  - `data_categories` (metadata-only | captions | figure bitmaps).
  - `endpoint` (local/offline/remote identifier).
  - `approved_at`, `expires_at` timestamps.
  - `approved_by` (user identifier or chat signature).
  - `notes` (text) summarizing prompt manifest.
- **Validation Rules**:
  - `expires_at` defaults to 30 days after approval; must be checked before reuse.
  - Remote operations require non-empty `endpoint` + logged manifest path.

## VisualizationDataset
- **Purpose**: Stores structured data powering concept maps, timelines, citation graphs, etc.
- **Fields**:
  - `dataset_id` (UUID), `type` (concept_map | timeline | citation_graph | backlog_chart ...).
  - `source_scope` (global/category IDs/paper sets).
  - `generated_from_snapshot` (AI-layer snapshot reference).
  - `data_path` (filesystem path pointing to JSON/CSV/graph files).
  - `last_updated_at`, `status` (current | stale | pending_regen).
- **Relationships**: Many `ReportManifest` entries can reference the same dataset if still `current`.
- **Validation Rules**:
  - Must mark `stale` if referenced papers/categories were modified since `generated_from_snapshot`.

## ShareBundleDescriptor
- **Purpose**: Metadata describing zipped or folder-based share artifacts.
- **Fields**:
  - `bundle_id` (UUID), `base_id`.
  - `manifest_id` reference (required).
  - `scope_description` (text) + included assets list.
  - `destination_path` and `format` (zip | directory).
  - `created_at`, `size_bytes`, `checksum`.
  - `include_visualizations` (bool), `include_figures` (bool), `notes`.
- **Validation Rules**:
  - Must reference exactly one `ReportManifest`.
  - `checksum` recomputed after bundle creation; share command fails if mismatch.

## Relationships Overview
- A `ReportBuildRequest` produces one `ReportManifest` on success.
- Each `ReportManifest` can feed multiple `ShareBundleDescriptor`s.
- `ConsentManifest` tokens are linked to build requests and stored within the manifest to prove approvals.
- `VisualizationDataset`s are reused across manifests until stale; regeneration flips status back to `current`.
