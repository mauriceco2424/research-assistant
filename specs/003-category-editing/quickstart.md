# Quickstart - Categorization & Editing Workflows

**Feature**: `003-category-editing`

## 0. Configure Categorization Defaults
1. Open `config/config.toml` inside your ResearchBase workspace (same directory that holds `AI/` and `User/`).
2. Add or edit the `[categorization]` table:
   ```toml
   [categorization]
   max_proposals = 5        # limit proposal cards per run
   timeout_ms    = 120000   # 2-minute clustering watchdog
   ```
3. Restart the app or re-run the CLI so the new limits apply to `categories propose`.

## 1. Generate Category Proposals
1. Ensure ingestion/metadata flows (Specs 001-002) are complete for the active Base.
2. In chat, run `categories propose` (optionally `--remote-summary on` to request AI narrative drafts).  
3. Review the proposal cards (name, confidence, sample papers) and accept, rename, or reject each one. Accepted categories are stored immediately and reports regenerate.

## 2. Edit Categories via Chat
1. Rename with `category rename "<old>" "<new>"`.  
2. Merge overlaps with `category merge "<name-a>" "<name-b>" --target "<new>"`.  
3. Split large categories using `category split "<name>" --by methodology` or another rule; confirm child categories before commit.  
4. Move specific papers using `category move-paper <paper-id> --to "<target>"` (supporting multiple IDs).

## 3. Maintain Narratives & Pins
1. Update narratives using `category narrative "<name>" --summary "..." --prompts "A;B"` to provide new summaries, learning prompts, or manual notes.  
2. Pass `--ai-approval "<consent text>"` when you want remote assistance; the consent manifest is logged per ResearchBase guidelines.  
3. Pin or unpin papers with either the narrative command (`--pin paper-id`) or `category pin <name> <paper-id> --add/--remove`; pinned entries float to the top of chat summaries and HTML category reports.  
4. Toggle figure galleries per category (`--gallery on/off`) to respect consented assets.

## 4. Monitor Category Health
1. Run `categories status` to see category counts, pinned highlights, staleness, and uncategorized backlog segments (grouped by venue/topic).  
2. Metrics are stored per Base so you can compare previous runs; review alerts for overloaded/stale categories and resolve with rename/merge/split actions before proceeding.

## 5. Undo & Audit
1. Use `category undo` to revert the most recent edit batch (rename, merge, narrative, move).  
2. Chat confirms the rollback and the orchestration history records who performed the action and which snapshot was restored.  
3. Regenerated HTML reports appear once the undo completes (<60 seconds for Bases ~1,000 papers).

## Storage Layout Reference
- AI-layer category artifacts live under `AI/<base-id>/categories/`.
- Subdirectories:
  - `definitions/` – accepted category definitions + narratives.
  - `assignments/` – paper-to-category links, refreshed after merges/splits.
  - `snapshots/` – undo checkpoints captured before each edit.
  - `proposals/` – latest proposal preview batches (confidence + sample papers).
- The structure is created automatically when a Base is provisioned; delete-with-care because undo/report regeneration depend on these files.





