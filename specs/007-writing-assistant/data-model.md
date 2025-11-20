# Writing Assistant Data Model

## 1. WritingProject (AI + User layers)
- **Fields**: `slug` (string, unique per Base), `title`, `description`, `baseId`, `owners[]`, `status` (Draft|Active|Review|Archived), `createdAt`, `updatedAt`, `defaultCompiler` (tectonic|pdflatex|custom path), `outlineId`, `activeBuildId`, `styleProfileVersion`, `referencedPaperIds[]`.
- **Relationships**: Links to WritingProfile (per Base) and OutlineNode tree via `outlineId`. Each project owns many DraftSection files + BuildSessions.
- **Validation**: Slug must be kebab-case, statuses follow Draft → Active → Review → Archived transitions, compiler path validated on creation/update.

## 2. WritingProfile (AI layer)
- **Fields**: `baseId`, `styleInterviewResponses`, `toneGuidelines`, `sectionOrdering`, `citationPreferences`, `styleModelIds[]`, `version`, `updatedAt`.
- **Relationships**: Many WritingProjects read from a single WritingProfile snapshot but may store overrides in their manifest.
- **Validation**: Interview responses stored with question ids + timestamps; styleModelIds must refer to StyleModel entries with consent tokens.

## 3. StyleModel (AI layer)
- **Fields**: `id`, `sourcePdfPath`, `analysisDate`, `features` (syntax fingerprints, length stats, citation density), `analysisMethod` (local|remote-provider-id), `consentToken` (optional), `notes`.
- **Relationships**: Attached to WritingProfile entries that include them.
- **Validation**: Remote analysis requires consent token + manifest reference.

## 4. OutlineNode (AI layer)
- **Fields**: `id`, `projectSlug`, `parentId`, `title`, `summary`, `references[]` (Paper IDs or report IDs), `status` (proposed|accepted|rejected|drafted), `order`, `lastEditedEventId`, `revisionHistory[]`.
- **Relationships**: Tree structure keyed by `parentId`; each accepted node maps to a DraftSection file path.
- **Validation**: Accept/reject transitions allowed only from `proposed`; deleting a node creates a tombstone entry to keep replay deterministic.

## 5. DraftSection (User + AI layers)
- **Fields**: `sectionId` (matches OutlineNode id), `filePath`, `lastGeneratedAt`, `lastEditedEventId`, `citations[]` (CiteKeys + Base Paper IDs), `undoChain[]` (references to stored diff hunks), `hash`.
- **Relationships**: Each DraftSection references OutlineNode + CitationLink entries; resides as `.tex` partial plus AI-layer metadata.
- **Validation**: File path must exist inside the project tree; `citations[]` must all resolve to Paper Base IDs unless flagged `UNVERIFIED`.

## 6. CitationLink (AI layer)
- **Fields**: `citeKey`, `paperId`, `status` (verified|needs_pdf|unverified), `lastCheckedAt`, `notes`.
- **Relationships**: Referenced from DraftSection entries; ties `.bib` entries back to Paper Base metadata.
- **Validation**: `citeKey` unique per project; status transitions only allowed in sequence verified ↔ needs_pdf ↔ unverified.

## 7. BuildSession (AI + User layers)
- **Fields**: `id`, `projectSlug`, `compiler`, `startedAt`, `completedAt`, `status` (success|failure), `logPath`, `pdfPath`, `errorSummary`, `inputs` (list of section ids + `.bib` version).
- **Relationships**: Linked from WritingProject `activeBuildId` and stored under `/builds/<timestamp>/`.
- **Validation**: `logPath` and `pdfPath` must remain inside the project tree; failure states require `errorSummary` + first offending file/line.

## 8. OrchestrationEvent (AI layer)
- **Fields**: `id`, `type`, `projectSlug`, `actor`, `timestamp`, `commandPayload`, `filesTouched[]`, `undoCheckpointPath`, `consentToken` (optional), `promptManifestPath` (optional), `status` (completed|failed).
- **Relationships**: Referenced by undo commands and compile/writing audit logs.
- **Validation**: Every mutating command MUST emit an event; `undoCheckpointPath` required for events that change drafts or outlines.
