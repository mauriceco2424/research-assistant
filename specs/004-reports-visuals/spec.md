# Feature Specification: Reports & Visualizations

**Feature Branch**: `004-reports-visuals`  
**Created**: 2025-11-19  
**Status**: Draft  
**Input**: User description: "Design the Reports & Visualizations feature set described in master_spec.md sections 3, 4, 10, and 11 (roadmap item Spec 04). Produce a single focused spec that turns AI-layer knowledge into regenerable HTML reports (category/global, figure galleries, visualizations) while honoring constitutional principles (P1-P10)."

## Clarifications

### Session 2025-11-19

- Q: Should report configuration (figures/visualizations toggles, asset filters) persist per Base or be specified every time? → A: Persist as Base defaults with per-command overrides.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Regenerate Base Reports Locally (Priority: P1)

The researcher issues `reports regenerate` (optionally with `--scope all` or a base/category filter) in chat and receives fresh HTML files for every category plus the global base summary under their local `/User/<Base>/reports/` directory. The chat response summarizes elapsed time, generated paths, asset counts, and orchestration IDs so they can audit or rerun jobs later.

**Why this priority**: Report regeneration is the first deliverable promised after ingestion/categorization (Specs 02-03). Without deterministic HTML output and audit trails, the Base provides no tangible value outside chat, blocking downstream sharing and writing workflows.

**Independent Test**: Triggering `reports regenerate --scope all` on a seeded Base should complete without errors and produce reproducible HTML + manifest artifacts even if no optional galleries/visualizations are enabled.

**Acceptance Scenarios**:

1. **Given** a Base with finalized categories and AI-layer narratives, **When** the user runs `reports regenerate --scope all`, **Then** new HTML files replace prior versions in `/User/<Base>/reports/` only after confirmation, and the chat reply lists each file path with timestamps and orchestration IDs.
2. **Given** a Base where only one category needs updating, **When** the user runs `reports regenerate --category "Neural Methods"`, **Then** only that category report and the global summary manifest are regenerated while other HTML files remain untouched yet listed as unchanged.
3. **Given** the user previously deleted the `/reports` folder manually, **When** they rerun `reports regenerate --scope all`, **Then** the system reprovisions the folder tree and regenerates every report using existing AI-layer inputs without prompting for missing assets unless optional galleries are requested.

---

### User Story 2 - Add Figures & Visualizations with Consent (Priority: P2)

While regenerating or configuring reports, the researcher enables figure galleries or visualization embeds. The system presents consent manifests describing which assets require figure extraction or remote LLM summarization, records approvals, and only then enriches the HTML with galleries, concept maps, timelines, or citation graphs.

**Why this priority**: Constitutional mandates (P1-P3) require explicit consent and regenerability when incorporating heavier assets that may involve figure extraction workflows or remote summarization calls.

**Independent Test**: Enable `reports configure --figures on --visualizations concept-map,timeline` and verify that consent dialogs appear, approvals are logged, assets are generated locally (or remote prompts are manifest-logged), and toggles can be reversed without corrupting AI-layer history.

**Acceptance Scenarios**:

1. **Given** figure extraction was previously disabled, **When** the user first enables a figure gallery, **Then** the system displays a consent manifest describing the files touched and stores the approval before generating thumbnails within the report bundle.
2. **Given** the user lacks network approval for remote summarization, **When** they request a visualization that requires LLM interpretation, **Then** the operation is paused with a consent prompt and does not call external endpoints until approval is captured.
3. **Given** the user turns off figure galleries mid-session, **When** they rerun report regeneration, **Then** new HTML files omit figure assets and the manifest notes that figures are excluded for that build.

---

### User Story 3 - Targeted Sharing & Bundling (Priority: P3)

The researcher shares selected reports via chat commands like `reports share --category "Neural Methods" --format zip --include-visualizations off`. The system packages only requested HTML files plus referenced assets, writes them to a local bundle, and attaches an audit manifest linking inputs so collaborators can reproduce the bundle on their own instance.

**Why this priority**: Researchers frequently deliver subsets of their work; controlled bundling with provenance is essential for verifying academic integrity and preventing accidental disclosure of unapproved assets.

**Independent Test**: Running `reports share --scope global --format zip` on any Base should output a zip file and audit manifest referencing the inputs and toggles used, without modifying the Base unless the user explicitly confirms overwriting an existing bundle.

**Acceptance Scenarios**:

1. **Given** a Global + Category report set exists, **When** the user shares only the global report, **Then** the bundle contains the HTML page, required CSS/assets, and a manifest enumerating AI-layer snapshots used, without including unrelated category folders.
2. **Given** a previous bundle exists at the target path, **When** the user runs `reports share ...` again, **Then** the system asks for confirmation before overwriting and logs the overwrite as an orchestration event.
3. **Given** certain visualizations require offline scripts the collaborator might not want, **When** the user toggles heavy assets off while bundling, **Then** the manifest records that visualizations were excluded and the zipped folder omits those datasets.

---

### Edge Cases

- Regeneration request arrives while a previous build is still running; subsequent commands must queue or prompt the user to cancel/continue to prevent interleaved writes.
- Local storage is low or `/User/<Base>/reports/` is read-only; system must stop before partial files are written and explain remediation steps in chat.
- Visualization datasets reference papers that were removed or recategorized; reports must gracefully flag missing data rather than embedding stale references.
- Consent manifests are older than configurable TTL; figure or visualization requests must reprompt for approval instead of reusing expired consent.
- Network connectivity drops after a user approves a remote summarization; the job must roll back to the last fully local build and log the failure.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Chat must expose `reports regenerate`, `reports configure`, and `reports share` commands (plus scoped flags) and describe required confirmations before execution.
- **FR-002**: Report generation must produce deterministic HTML, CSS, and asset folders under `/User/<Base>/reports/` and warn before overwriting existing artifacts.
- **FR-003**: Commands must accept scopes (`--scope all`, `--category`, `--global-only`, `--include <list>`) so users can regenerate or share specific subsets without touching unrelated HTML files.
- **FR-003a**: Each Base must store default report configuration (figures/visualization toggles, excluded assets), apply those defaults automatically on future commands, and allow users to override them per invocation without altering saved defaults unless explicitly updated.
- **FR-004**: Every job must emit orchestration events (start/end timestamps, scope, duration, asset counts, success/failure) and surface their IDs in chat.
- **FR-005**: Each build must create or update a machine-readable audit manifest (JSON or Markdown) summarizing inputs: AI-layer category snapshot IDs, metrics revision, visualization datasets, consent status, and toggles used.
- **FR-006**: Figure galleries and visualization embeds must remain OFF by default and only turn on after the user consents to the precise manifest describing assets, external prompts, and retention behaviors.
- **FR-007**: When figure galleries are enabled, the system must extract images locally, store them under `/User/<Base>/reports/assets/figures/<category>/`, and update manifests without modifying AI-layer history beyond referencing asset locations.
- **FR-008**: When visualizations require remote summarization or layout (e.g., concept maps), the workflow must generate a prompt manifest, request explicit approval, and log endpoint, prompt summary, and response provenance per P2.
- **FR-009**: Users must be able to toggle each visualization type (concept maps, timelines, citation graphs, backlog charts) individually, and builds must respect the last confirmed configuration.
- **FR-010**: `reports share` must assemble only requested HTML + assets into a user-specified output path (folder or zip) without copying Base files elsewhere or invoking non-local storage.
- **FR-011**: Bundled outputs must include a provenance summary referencing the audit manifest and any consent manifests so recipients can reproduce the bundle.
- **FR-012**: If a requested scope references stale or missing AI-layer data, the command must stop with guidance (e.g., "rerun categorization") instead of emitting partially populated HTML.
- **FR-013**: Regeneration for Bases up to 1,000 papers must complete within 60 seconds on reference hardware, streaming progress messages (percent complete, component counts) to chat at least every 5 seconds.
- **FR-014**: Errors (disk full, missing consent, network refusal) must fail fast, leave existing HTML untouched, and include actionable remediation steps plus links to orchestration logs.

### Key Entities *(include if feature involves data)*

- **ReportBuildRequest**: Captures the user command, scope, toggles, and timestamp; used to enqueue or serialize report generation jobs and tie chat messages to orchestration IDs.
- **ReportManifest**: JSON/Markdown artifact stored per build containing AI-layer snapshot refs, asset hashes, consent tokens, visualizations included, and output file paths so identical manifests reproduce identical HTML.
- **ConsentManifest**: Describes figure extraction or remote summarization work (purpose, assets, endpoint, expiry) and records the user's approval with timestamp and Base context.
- **VisualizationDataset**: Structured representation of concept maps, timelines, citation graphs, etc., referencing their source papers/categories and storage location for reuse across builds.
- **ShareBundleDescriptor**: Metadata describing user-requested bundles (scope, destination, include/exclude toggles, checksum) so zipped exports can be audited or regenerated later.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 95% of `reports regenerate --scope all` jobs on Bases <=1,000 papers finish within 60 seconds and surface orchestration IDs + output paths in chat without manual file hunting.
- **SC-002**: 100% of figure gallery or visualization requests display consent manifests before proceeding, and approval logs reference manifests traceable to the resulting HTML bundles.
- **SC-003**: At least 90% of targeted bundle requests (`reports share` with filters) complete on the first attempt, producing bundles that contain only requested assets and validated manifests.
- **SC-004**: Independent reviewers can take any audit manifest and reproduce identical HTML assets (byte-identical aside from timestamps) in >=95% of tests, validating regenerability.
- **SC-005**: User satisfaction surveys for reporting flows show >=4/5 rating for clarity of chat guidance and orchestration transparency over two release cycles.

## Assumptions

- AI-layer narratives, metrics, and visualization datasets already exist from Specs 02-03 and remain accessible when this feature runs.
- Users have sufficient local disk space for at least two complete sets of reports; low-disk handling only needs to warn/abort, not auto-clean older versions.
- Reference hardware assumption aligns with prior specs (desktop-class CPU, local storage SSD); performance targets will be validated on that baseline.
- Consent manifests expire after a governance-defined period (default 30 days) to keep figure usage compliant; this timeline can be adjusted in configuration without spec changes.
