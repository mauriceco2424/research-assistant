# Spec Argument – Reports & Visualizations (Spec 04)

Design the **Reports & Visualizations** feature set described in `master_spec.md` §§3, 4, 10, and 11 (roadmap item Spec 04). Produce a single focused spec that turns AI-layer knowledge into regenerable HTML reports (category/global, figure galleries, visualizations) while honoring constitutional principles (P1–P10).

## User Intent & Scenarios
- After ingestion (Spec 02) and categorization/editing (Spec 03), the researcher wants commands like `reports regenerate`, `reports configure`, or `reports share` to produce local HTML artifacts they can open or distribute without manual file wrangling.
- Category reports must include narratives, pinned highlights, backlog/health metrics, optional figure galleries (with consent), and embedded visualizations (concept maps, timelines, citation graphs) per `master_spec.md` §11.
- Global reports summarize the entire Base (counts, discovery prompts, writing hooks) and expose toggles for heavier assets so offline users can exclude figures/scripts.
- Whenever figure extraction or remote summarization is requested, the workflow must prompt for consent manifests and log approvals before touching external endpoints.

## Constraints & Constitutional Alignment
- **P1 Local-First & P2 Consent**: Report generation, figure assets, and visualization data stay on the local filesystem. Any remote LLM assistance explicitly lists prompt manifests and requires user approval each time.
- **P3/P4 Dual-Layer & Regenerability**: HTML outputs are deterministic derivatives of AI-layer sources (categories JSON, narratives, metrics, visualization datasets). Users can delete/rebuild them at any time and expect identical content when sources are unchanged.
- **P5 Chat-First**: No new persistent UI panes; all report actions originate in chat with clear progress + completion summaries (file paths, durations, warnings).
- **P6 Transparency**: Long-running report jobs emit orchestration events (start/end timestamps, success/failure, asset counts) and require confirmation before overwriting previous outputs.
- **P7 Academic Integrity**: Reports cite referenced papers, label AI-generated narratives, and distinguish suggested vs. verified figures/visualizations.
- **Performance**: Regenerating category + global reports for Bases ≤1 000 papers must finish ≲60 s (per plan/perf targets) with progress copy in chat.

## Success Criteria
1. Running `reports regenerate --scope all` produces fresh category and global HTML files under `/User/<Base>/reports/`, embedding narratives, pinned highlights, backlog stats, and optional visualizations; chat replies with file paths, durations, and orchestration IDs.
2. Enabling figure galleries triggers consent prompts, stores approvals (e.g., `figure_gallery_render` manifest), extracts images locally, and embeds them without touching AI-layer history; disabling galleries omits figure assets from new builds.
3. Users can target specific outputs (e.g., `reports regenerate --category "Neural Methods"` or `reports share --format zip`) and the system bundles only the requested HTML + assets while logging provenance manifests so partners can verify inputs.
4. Each report bundle references an audit manifest (JSON/Markdown) in the AI layer that enumerates inputs (category snapshot, metrics revision, visualization data) so re-running the same manifest reproduces identical HTML.

## Scope Guardrails
- Focus strictly on HTML reports, figure galleries, and embedded visualizations. Do **not** expand ingestion, acquisition, writing assistant, or UI shell behavior; those belong to other specs.
- Reference `.specify/memory/constitution.md` for P1–P10 and call out any potential conflicts (e.g., external visualization libraries) with mitigation guidance.
- Avoid implementation details (filenames, functions). Define behaviors, consent flows, toggles, and performance/telemetry expectations so `/speckit.plan` and `/speckit.tasks` can translate requirements into architecture.