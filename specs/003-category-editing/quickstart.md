# Quickstart – Categorization & Editing Workflows

**Feature**: `003-category-editing`

## 1. Generate Category Proposals
1. Ensure ingestion/metadata flows (Specs 001–002) are complete for the active Base.
2. In chat, run `categories propose` (optionally `--remote-summary on` to request AI narrative drafts).  
3. Review the proposal cards (name, confidence, sample papers) and accept, rename, or reject each one. Accepted categories are stored immediately and reports regenerate.

## 2. Edit Categories via Chat
1. Rename with `category rename "<old>" "<new>"`.  
2. Merge overlaps with `category merge "<name-a>" "<name-b>" --target "<new>"`.  
3. Split large categories using `category split "<name>" --by methodology` or another rule; confirm child categories before commit.  
4. Move specific papers using `category move-paper <paper-id> --to "<target>"` (supporting multiple IDs).

## 3. Maintain Narratives & Pins
1. Update narratives using `category narrative "<name>"` to provide new summaries, learning prompts, or manual notes.  
2. Pin or unpin papers inside the same command; pinned entries float to the top of chat summaries and HTML category reports.  
3. Toggle figure galleries per category to respect consented assets.

## 4. Monitor Category Health
1. Run `categories status` to see counts, pinned highlights, staleness, and uncategorized backlog.  
2. Follow the suggested quick actions (auto-cluster backlog, open merge wizard, etc.) to resolve flagged issues before proceeding.

## 5. Undo & Audit
1. Use `category undo` to revert the most recent edit batch (rename, merge, narrative, move).  
2. Chat confirms the rollback and the orchestration history records who performed the action and which snapshot was restored.  
3. Regenerated HTML reports appear once the undo completes (<60 seconds for Bases ≤1,000 papers).
