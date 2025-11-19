# Research Notes – Reports & Visualizations (Spec 04)

## Decision 1: Persist report configuration per Base with override flags
- **Rationale**: Users should not repeat consented choices (figures, viz toggles) each run, reducing error risk while still allowing per-command overrides.
- **Alternatives considered**:
  - *Stateless commands*: increases chat friction and risk of mismatched bundles.
  - *Named profiles*: added management surface contradicts chat-first minimalism.

## Decision 2: Use manifest pair (ReportManifest + ConsentManifest) as audit backbone
- **Rationale**: Aligns with P3/P4 by referencing AI-layer snapshot IDs, consent tokens, asset hashes, and config signatures so HTML can be regenerated verbatim.
- **Alternatives considered**:
  - Embed metadata directly inside HTML: harder to diff/version and violates AI-layer separation.
  - Store per-run SQLite records: introduces unnecessary dependency and obscures transparency.

## Decision 3: Progressive orchestration logging with <=5s updates
- **Rationale**: Keeps long-running report jobs observable and cancellable, satisfying P5/P6 transparency requirements while hitting 60s completion target.
- **Alternatives considered**:
  - Single final message: gives no feedback during heavy runs.
  - UI progress bar: conflicts with chat-first constraint.

## Decision 4: Bundle creation limited to requested assets + provenance summary
- **Rationale**: Prevents accidental disclosure of unapproved artifacts and ensures recipients have the manifest needed to reproduce or verify contents.
- **Alternatives considered**:
  - Always include entire `/reports` tree: bloats artifacts and risks sharing private categories.
  - Custom share UI: exceeds scope and duplicates bundling logic.

## Decision 5: Consent manifests expire after configurable TTL (default 30 days)
- **Rationale**: Satisfies academic compliance expectations by forcing periodic reconfirmation for figure/remote summarization use.
- **Alternatives considered**:
  - Never expire: risks stale approvals violating consent principles.
  - Require consent every run: deteriorates UX and slows workflows.

## Phase 6 Validation & Analytics Addendum

- **Dry-run verification (T028)**  
  - Executed the full quickstart flow on a seeded Base using `reports configure`, `reports regenerate`, `reports share`, and `verify_manifest`.  
  - Observed orchestration events (`ReportsGenerated`, `ReportsShared`) with matching consent tokens and manifest IDs in `AI/<Base>/events.jsonl`.  
  - Confirmed share manifests landed under `AI/<Base>/reports/share_manifests/` and `verify_manifest` reported zero hash drift, satisfying SC-004 reproducibility checks.

- **SC-005 instrumentation plan (T029)**  
  - Tag every `ReportsGenerated` / `ReportsShared` event with `include_figures`, `include_visualizations`, durations, and bundle sizes to feed satisfaction dashboards.  
  - Capture chat-surface summaries (handlers/report_updates) including consent IDs so post-run surveys can correlate user perceptions with logged transparency events.  
  - Weekly script will aggregate orchestration logs (counts, durations, consent refresh churn) and feed the governance learning review noted in the master spec.
