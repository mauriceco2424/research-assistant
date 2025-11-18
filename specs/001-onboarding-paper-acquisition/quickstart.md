# Quickstart – Onboarding & Paper Acquisition Workflow

**Feature**: `001-onboarding-paper-acquisition`  
**Spec**: `specs/001-onboarding-paper-acquisition/spec.md`

This quickstart describes the high-level flow for onboarding and paper
acquisition in ResearchBase, as seen from an end user’s perspective.

---

## 1. Create and Select a Paper Base

1. Start ResearchBase.
2. When prompted, create a new Paper Base by giving it a descriptive name
   (e.g., “ML & AI Base”).
3. The app sets up local directories for the Base’s User Layer and AI Layer.
4. If you have multiple Bases, choose which one to work in from the startup
   Base list or via chat (e.g., “Switch to my Neuroscience Base”).

---

## 2. Onboard with Existing PDFs (Path A)

1. In chat, say something like: “I already have PDFs to import.”
2. Select a folder of PDFs or a supported library export file when prompted.
3. The app ingests the documents into your active Base:
   - extracts metadata and text locally,
   - updates AI-layer records.
4. The AI proposes initial categories based on your library; confirm, rename,
   merge, or split them via chat.
5. Ask the AI to “generate category reports and a global report” to produce
   HTML summaries you can open in your browser.

No PDFs are downloaded in this flow; it only uses files you already have.

---

## 3. Onboard without PDFs (Path B)

1. In chat, say: “I don’t have papers yet—help me build a base.”
2. Answer a short interview about your research area, questions, and level of
   expertise.
3. The AI returns a list of candidate papers with titles, metadata, and DOIs
   (no downloads yet).
4. Select the candidates you want (e.g., “Add these 8 papers”) and confirm
   when the app asks to start acquisition.
5. The app:
   - resolves metadata from DOIs/IDs and
   - tries to fetch open-access PDFs when permitted.
6. Papers with retrieved PDFs are added as full library entries; others
   become metadata-only entries marked `NEEDS_PDF`.
7. The AI tells you which papers need manual download and attachment.

---

## 4. Discover and Add New Papers Later

1. From any Base, ask the AI to “find new papers on [topic]” or “fill gaps in
   category [name].”
2. The AI proposes candidate papers with metadata and DOIs/IDs.
3. Select and approve a batch to add (e.g., “Add these 5 and fetch PDFs”).
4. The same Paper Acquisition Workflow runs:
   - no downloads happen until you explicitly approve a batch;
   - successes attach PDFs; failures become `NEEDS_PDF` entries.

---

## 5. Review Acquisition History and Undo

1. At any time, ask: “Show my recent acquisition history.”
2. The app lists recent batches, including:
   - when they ran,
   - how many papers were added,
   - which papers still need PDFs.
3. To undo the most recent batch, say: “Undo the last acquisition batch for
   this Base.”
4. The app removes library entries and derived artifacts from that batch while
   leaving other Bases and earlier batches unchanged.

