# Spec Argument – Paper Ingestion, Metadata, and Figure Extraction

Design the **Paper Ingestion & Figure Extraction** system for ResearchBase (Spec 02), building on the existing multi-Base onboarding and acquisition flows. The specification must cover:

1. **Robust ingestion pipeline** for local PDFs and metadata updates:
   - High-confidence PDF parsing (text, references, keywords, sections) with resumable batches and progress reporting.
   - Metadata normalization and enrichment (DOI/Crossref/Unpaywall lookup), dedup detection, and a way to re-ingest/update existing entries.
   - Explicit handling of corrupt/missing PDFs, multi-language documents, and split/merged papers, with chat-first recovery workflows.

2. **Figure and visual extraction**:
   - Optional figure extraction (images + captions) per ingestion batch, gated by explicit consent (per P2/P6) and stored under the User Layer.
   - Linking figures to their source papers and AI-layer summaries; ability to include figure galleries in category/global reports and to request figure interpretations via chat.

3. **Enrichment and validation**:
   - Structured AI-layer artifacts for summaries, references, keywords, figures, and ingestion history so reports remain regenerable (P3/P4).
   - Quality checks for metadata conflicts, missing DOIs, and figure extraction failures, with chat-based resolutions.

4. **Chat-first UX & orchestration**:
   - Conversational ingestion commands (start, pause, resume, status) with progress messages and error explanations.
   - Chat commands to reprocess specific papers, re-run figure extraction, or enrich metadata.

5. **Governance & consent**:
   - All figure extraction and remote metadata lookups must follow the constitution’s consent/logging rules (P1–P6), including per-batch approval, manifest logging, undo, and local-only storage by default.

Ensure the spec stays aligned with `master_spec.md` and `.specify/memory/constitution.md`, defines measurable success criteria, and outlines how figures/metadata enhancements feed into HTML reports and future learning/writing modes.

