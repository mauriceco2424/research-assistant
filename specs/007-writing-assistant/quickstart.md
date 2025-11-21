# Writing Assistant Quickstart

1. **Prepare environment**
   - Ensure Base directory exists with ingested papers (Specs 01–06 complete).
   - Install `tectonic` (preferred) or provide a `pdflatex` path via settings.
   - Confirm `.specify` orchestration tools (prompt manifests, consent logs) are writable.

2. **Start a writing project**
   - In chat, run `/writing start "Survey on multimodal alignment"`.
   - Answer style interview prompts (tone, venue, evidence expectations).
   - Slug is auto-generated and collision-safe; orchestration logs record `project_created`.
   - Verify `/User/<Base>/WritingProjects/survey-on-multimodal-alignment/` is created with `project.json`, `main.tex`, `sections/`, `.bib`, and AI-layer outline metadata.

3. **Ingest style models**
   - Upload 1–3 favorite PDFs via chat.
   - Confirm local analysis completes; if remote inference is requested, approve consent and see orchestration event reference.

4. **Outline + draft**
   - Ask "Generate an outline"; accept or rewrite nodes via chat commands. Accepted nodes create undo checkpoints and push placeholder citations into `references.bib`.
   - Request "Draft the introduction referencing ingestion highlights"; drafts are stored under `/sections/<outlineNodeId>.tex` with AI-layer metadata in `ai_layer/writing/<slug>/draft_sections/`.
   - If you added citations by hand, rerun draft to see drift warnings when `references.bib` diverges from outline metadata.

5. **Inline edit & cite**
   - Highlight or reference section ID (e.g., `sec-intro`) and request "tighten this and cite Smith 2021".
   - Confirm diff summary + undo token returned; verify `.tex` reflects citation and `UNVERIFIED` markers when necessary.
   - Use `/writing projects <slug> undo <eventId>` to restore the previous checkpoint written to `ai_layer/writing/<slug>/undo/`.

6. **Compile locally**
   - Run "compile the writing project".
   - Inspect `/builds/<timestamp>/` for `main.pdf` + logs; chat echoes key warnings/errors pulled from logs.
   - If no drafts exist, the compile is blocked with guidance to generate sections first.

7. **Undo / audit**
   - Use "undo event <id>" to revert a recent change.
   - Check `ai_layer/orchestration/<event>.json` for transparency (prompt manifests, consent tokens).

8. **Archive or continue**
   - Once ready to finalize, set project status to Review then Archived via chat commands.
   - Validate `project.json` status transitions and that archived projects remain regenerable.

**E2E check (latest run)**: Verified chat flow through start → outline → draft → edit/undo → compile using placeholder compiler output. Follow-up: swap stub compile with real `tectonic` binary when available on the machine.
