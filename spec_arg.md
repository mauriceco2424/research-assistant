# Spec Argument – Categorization & Editing Workflows (Spec 03)

Design the **Categorization & Editing** feature set for ResearchBase, the next roadmap item after Spec 02 per `master_spec.md §16`. This request should produce one focused spec that gives researchers a chat-first way to organize newly ingested papers, revise category structures, and keep HTML reports/AI-layer artifacts aligned without violating P1–P10.

## User Intent & Scenarios
- After completing ingestion/metadata (Spec 02), the researcher wants the AI to propose initial categories and summaries, accept/modify those suggestions, and keep the Base organized as new papers arrive.
- They need to merge or split categories, rename them, move papers between categories, and annotate category narratives entirely via chat, with those edits reflected both in AI-layer metadata and regenerable HTML reports.
- They want quick visibility into category health (paper counts, uncategorized backlog, pinned/highlighted papers) and lightweight per-category editing like notes or priority flags that downstream learning/reporting modes can reuse.

## Constraints & Constitutional Alignment
- **P1 Local-First**: All category models, summaries, and edits remain in the Base’s local User/AI layers. Remote LLM assistance (if needed) must present a consent manifest per P2 before using external endpoints.
- **P3/P4 Dual-Layer & Regenerability**: Category definitions, paper assignments, and narratives must be stored in AI-layer JSON/Markdown so HTML category/global reports stay regenerable without rerunning ingestion. Editing commands must update both layers consistently.
- **P5 Chat-First**: No new complex UI panes—category management happens in chat with optional HTML report previews.
- **P6 Transparency/Undo**: Auto-cluster, merge, split, move, and rename operations emit orchestration events and support undo for at least the last operation per Base.
- **P7 Academic Integrity**: Category summaries cite their source papers or clearly mark unverified insights.

## Success Criteria
1. From a freshly ingested Base, running a command like `categories propose` produces ≥5 suggested categories with confidence scores, representative papers, and short summaries persisted under AI-layer `categories/*.json` plus matching entries in the User-layer reports.
2. Chat commands exist for rename/merge/split/move/pin/unpin operations, and executing them updates AI-layer metadata, regenerates HTML category/global reports within 60 s for Bases ≤1 000 papers, and surfaces confirmation in chat.
3. Undoing the last category edit restores the previous AI-layer snapshot and re-renders reports; orchestration logs capture each edit with timestamps, actor, and affected categories.
4. `categories status` (or similar) lists all categories with paper counts, uncategorized backlog, pinned papers, and outstanding TODOs, enabling researchers to see gaps at a glance.

## Scope Guardrails
- Stick to categorization + editing workflows. Defer new ingestion, acquisition, or desktop-shell UI work to future specs on the roadmap.
- Reference `master_spec.md §§5.2–5.3 & 16` for context, plus `.specify/memory/constitution.md` for P1–P10 alignment. Call out any potential conflicts explicitly in the spec.
- Do not prescribe module/file names or implementation details beyond what’s necessary for requirements; leave structure decisions for `/speckit.plan` and `/speckit.tasks`.
