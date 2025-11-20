# Writing Assistant Lifecycle Commands

This document captures the high-level chat commands introduced by the Writing Assistant feature so new teammates can follow the project flow without re-reading the full spec.

## Project Lifecycle
- `/writing start "<title>"` – scaffolds a project directory, runs the style interview, records consent, and emits `project_created` events.
- `/writing projects` – lists known projects with their status; allows selecting one as the active context.
- `/writing projects {slug}` PATCH/Archive – updates status, default compiler, or archives completed work while preserving regenerability.

## Style + Outline Loop
- `/writing projects {slug} style-interview` – replays the interview when tone guidance needs refreshing.
- `/writing projects {slug}/outline` commands – request outlines, accept nodes, and checkpoint undo history (`outline_created`/`outline_modified`).
- `/writing projects {slug}/drafts` – generates drafts tied to accepted outline nodes and syncs `.tex` files.

## Inline Edits & Compilation
- `/writing edit` requests apply structured diffs, inject citations, and emit undo checkpoints (`section_edited`).
- `/writing undo` reverts by orchestration event id using stored AI-layer checkpoints.
- `/writing compile` runs the configured local compiler (preferring `tectonic`, falling back to `pdflatex`), streams logs, and stores PDFs + events (`compile_attempted`).

## Notes
- All commands are chat-first and respect local-first privacy guarantees.
- Remote style analysis or citation lookups must surface prompt manifests + consent tokens before running.
- Build artifacts live under `/builds/<timestamp>/` inside each project to keep regenerability intact.
