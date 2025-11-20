# Writing Assistant Research

## Decision 1: Default Local LaTeX Tooling
- **Decision**: Use the `tectonic` CLI as the default compiler with `pdflatex` fallback plus log streaming wrappers.
- **Rationale**: Tectonic ships as a single binary, works cross-platform without a full TeXLive install, supports incremental caching, and exposes structured log output that we can parse for file/line annotations. A fallback to any user-provided `pdflatex` path satisfies researchers who already rely on classic toolchains. Both options run locally and honor P1.
- **Alternatives Considered**:
  - `latexmk`: Provides automation but still depends on TeXLive footprint; heavier to bundle.
  - Cloud compilation services: Rejected because they violate P1/P2 and add network consent complexity for every build.

## Decision 2: Undo/Redo Strategy without Git
- **Decision**: Store deterministic undo checkpoints inside the AI layer by persisting (a) outline JSON snapshots, (b) diff hunks for `.tex` edits, and (c) the orchestration event metadata linking both. Git integration remains optional, but revert commands operate purely via stored checkpoints.
- **Rationale**: Guarantees undo availability even when git is disabled or when users edit files via external editors. Snapshots remain lightweight because outline nodes + diff hunks are text-based JSON/patch files, keeping P3/P4 intact.
- **Alternatives Considered**:
  - Requiring git for undo: Rejected because some Bases may not use git and it would block compliance with P6.
  - Binary backup archives: Harder to diff/inspect; conflicts with regenerability goals.

## Decision 3: Style Model Feature Extraction
- **Decision**: Perform PDF parsing + embedding locally using existing ingestion pipelines (Spec 02) plus lightweight stylistic heuristics (sentence length, citation density). Only if the user opts in do we send anonymized excerpt chunks to a remote LLM; such calls emit prompt manifests + consent tokens.
- **Rationale**: Reuses proven local code paths, guarantees privacy by default, and still allows advanced analysis under explicit consent per P2.
- **Alternatives Considered**:
  - Always using remote LLMs: Violates P1/P2 defaults and introduces approval fatigue.
  - Skipping style models: Would fail to meet spec requirements for personalized drafting.
