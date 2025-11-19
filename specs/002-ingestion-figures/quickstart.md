# Quickstart – Paper Ingestion, Metadata Enrichment & Figure Extraction

**Feature**: `002-ingestion-figures`

## 1. Start Ingestion
1. In chat, run `ingest start` and provide the local folder/export path.
2. Watch progress updates (processed counts, ETA). Use `ingest pause` / `ingest resume` as needed.
3. If the app restarts, type `ingest status` to continue from the last checkpoint. Tune chunk size and concurrency via `config.toml` (`ingestion.checkpoint_interval_files`, `ingestion.max_parallel_file_copies`) for faster local SSDs.

## 2. Review Failures & Skipped Files
1. After completion, the chat summary lists skipped/corrupt files with reasons.
2. Use `ingest retry <file>` or manually fix and rerun ingestion for those items.

## 3. Metadata Enrichment & Dedup
1. Run `metadata refresh` to normalize DOIs, authors, keywords.
2. Resolve duplicate suggestions via chat prompts (merge/keep) and confirm.
3. For individual papers, run `metadata refresh <paper-id>` to preview diffs before applying.

## 4. Figure Extraction (Optional)
1. After ingestion, run `figures extract <batch-id>`.
2. The system summarizes required data (e.g., remote lookups) and asks for approval.
3. Upon approval, figures are saved under `User/<Base>/figures/<batch>` and linked to reports.

## 5. Reports & Galleries
1. Run `reports regenerate --figures on/off` to update category/global reports with or without galleries.
2. Open the HTML reports from the User Layer or via chat links.

## 6. Audit & Undo
1. `history show 7d` lists ingestion/figure batches with timestamps and statuses.
2. `undo last ingestion` or `undo last figure extraction` removes newly added files and metadata for the most recent batch (per Base).

## 7. Metrics & Performance
1. Ingestion completion events now log duration + SLA warnings; review `AI/<Base>/metrics.jsonl` for granular metrics.
2. Figure extraction metrics include success rates so you can confirm how many assets were captured per batch.
3. Report regeneration entries include timing data; runs exceeding 60 seconds surface a chat warning for follow-up tuning.
