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
   - Ask "Generate an outline"; accept or rewrite nodes via chat buttons/commands.
   - Request "Draft the introduction referencing ingestion highlights"; verify `.tex` partial and AI-layer metadata update together.

5. **Inline edit & cite**
   - Highlight or reference section ID (e.g., `sec-intro`) and request "tighten this and cite Smith 2021".
   - Confirm diff summary + undo token returned; verify `.tex` reflects citation and `UNVERIFIED` markers when necessary.

6. **Compile locally**
   - Run "compile the writing project".
   - Monitor streamed log output; inspect `/builds/<timestamp>/` for `main.pdf` + logs.
   - On errors, follow chat guidance referencing file/line numbers.

7. **Undo / audit**
   - Use "undo event <id>" to revert a recent change.
   - Check `ai_layer/orchestration/<event>.json` for transparency (prompt manifests, consent tokens).

8. **Archive or continue**
   - Once ready to finalize, set project status to Review then Archived via chat commands.
   - Validate `project.json` status transitions and that archived projects remain regenerable.
