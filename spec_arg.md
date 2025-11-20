# Spec Argument - Writing Assistant (Spec 07)

Design the **Writing Assistant (LaTeX)** described in `master_spec.md` §8 and roadmap item **Spec 07**. Produce a feature spec that turns the chat-first environment into a local-first co-authoring space: style interviews, outline + draft generation, `.tex/.bib` lifecycle, inline edits, citation injection from the Base, and PDF compilation. The spec must respect the ResearchBase constitution (P1–P10) and build atop prior specs (Spec 05 profiles, Spec 06 intent router).

## User Intent & Scenarios
- **Style Interview & Setup**: Researcher starts “Help me write a survey on multimodal alignment.” Assistant runs a writing-style interview (leveraging WritingProfile + favorite papers), stores preferences, and scaffolds `/User/<Base>/WritingProjects/<slug>/`.
- **Outline ➜ Draft ➜ Iteration**: User asks for an outline, accepts sections, then says “Draft the intro and weave in the last 3 ingestion highlights.” Assistant generates LaTeX sections referencing category summaries and cites Base papers via `.bib`.
- **Inline Chat Edits**: Researcher highlights a paragraph (or references section ID) and requests “tighten this paragraph and add a citation for Smith 2021.” Assistant edits the `.tex`, logs diffs, and reports changes with undo pointers.
- **Build & Preview**: User runs “compile the writing project” and receives PDF build feedback, including LaTeX errors surfaced in chat with file/line context. Builds occur locally; no remote compilation.
- **Style Model Ingestion**: Researcher points to favorite PDFs to use as style models. Assistant analyzes them locally (no cloud upload) and updates WritingProfile metadata, warning if remote inference would be needed and requesting consent.

## Constraints & Constitutional Alignment
- **P1 Local-First / P2 Consent**: All project files live under `/User/<Base>/WritingProjects/`. Any optional remote inference (e.g., style analysis via LLM) requires manifest summaries, explicit approval, and logged consent tokens.
- **P3/P4 Dual-Layer**: LaTeX artifacts belong to the User layer. AI-layer stores structured writing metadata (outline JSON, revision history, prompt manifests) so drafts can be regenerated. Chat edits must reference deterministic payloads for replay.
- **P5 Chat-First / Minimal UI**: Every interaction (create project, edit section, compile, view logs) flows through chat. No bespoke editors; instead, assistant surfaces snippets, diff summaries, and file paths for manual editing if desired.
- **P6 Transparency & Undoability**: Editing commands reply with change summaries, event IDs, and instructions to revert via git-like backups or AI-layer checkpoints. Compilations show logs so users see exactly what happened.
- **P7 Integrity / P8 Learning**: Citations must map to actual Base entries—no hallucinated references. When the assistant can’t confirm a citation, it labels it “UNVERIFIED” in draft + chat. Writing guidance references KnowledgeProfile + WritingProfile evidence.
- **P9/P10 Versioning & Extensibility**: Writing project metadata, outline schema, and style interviews must be versioned so future specs (Learning Mode, multi-author workflows) can extend them without migrations.

## Success Criteria
1. **Project Lifecycle**: Define how writing projects are created, listed, switched, and deleted (folder layout, config JSON, required files). Include slug rules and Base scoping.
2. **Style Interview & Profiles**: Specify the interview prompts, how results update WritingProfile, how favorite papers become style models, and how remote consent is handled when analysis leaves the device.
3. **Outline & Draft Generation**: Detail the JSON outline schema, how it maps to `.tex` sections, how users accept/reject sections via chat, and how the assistant references Base artifacts (reports, profiles, papers).
4. **Inline Editing & Citations**: Describe the command set for editing (insert, rewrite, cite, summarize). Include citation resolution flow, failure handling, undo checkpoints, and AI-layer logging requirements.
5. **Compilation Workflow**: Outline how the assistant invokes local LaTeX (configurable path), streams logs/errors back to chat, and stores build artifacts. Address environment configuration, retries, and how builds respect local-only policy.
6. **Safety & Auditability**: Enumerate orchestration events emitted for writing operations (outline_created, draft_generated, section_edited, compile_attempted) with required metadata (project id, files touched, consent ids if any).

## Scope Guardrails
- Focus on LaTeX projects only (no Markdown editor, no collaborative multi-user features yet).
- Do not implement a full diff viewer UI; limit to chat summaries + file paths.
- Remote ops limited to optional style analysis; everything else (drafting, citation lookup, compile) must run locally.
- Assume Spec 06 router dispatches writing commands; this spec defines the capabilities, storage, and behaviors that commands invoke.
