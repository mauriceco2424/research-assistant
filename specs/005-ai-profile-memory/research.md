# Phase 0 Research - AI Profiles & Long-Term Memory

## 1. Profile Artifact Schema

- **Decision**: Represent each profile as stable, human-readable JSON with deterministic key ordering, storing `metadata` (id, last_updated, scope flags), `summary` (high-level bullets), and `fields` (ordered map of domain-specific attributes). KnowledgeProfile entries embed an array of `{concept, mastery_level, evidence_refs[], weakness_flags[], learning_links[]}` records.
- **Rationale**: Deterministic JSON keeps git diffs readable, satisfies P3 diff-friendly requirement, and lets regeneration hash comparisons stay reliable. Separating metadata, summary, and fields mirrors how chat responses surface data (structured block plus narrative) while allowing interviews to patch only relevant sections.
- **Alternatives considered**: (a) Nested YAML - easier for comments but inconsistent ordering and higher parsing overhead; (b) single flat JSON document - harder to diff and to deliver user-facing summaries without duplicating derived fields.

## 2. Orchestration Event & Consent Logging

- **Decision**: Extend the existing orchestration event struct with `profile_type`, `change_kind`, `diff_summary`, `undo_token`, and `consent_manifest_ids`. Consent manifests live as separate JSON files keyed by manifest id under `/AI/<Base>/consent/manifests`, and events store references plus prompt metadata (operation type, data categories).
- **Rationale**: Keeps the orchestration log aligned with other features, satisfies P2/P6 by making every remote call auditable, and lets `profile audit` run as a pure log query. Storing manifests separately avoids inflating profile JSON with consent details while still linking them deterministically.
- **Alternatives considered**: (a) Embed consent manifest payloads directly in profile files - bloats primary artifacts and risks leaking sensitive prompts when exporting; (b) rely on an external database for orchestration events - violates P1/P3.

## 3. Export & Regeneration Strategy

- **Decision**: Use `sha2` hashes to snapshot each profile JSON after writes, store alongside orchestration event ids, and have `profile regenerate --from-history` replay events chronologically, verifying the final hash before replacing live files. Exports bundle current JSON plus HTML summaries and a trimmed audit log into ZIP archives guarded by per-profile filesystem locks.
- **Rationale**: Hash verification proves regenerability (P4) and immediately surfaces missing/corrupted logs. Locks avoid torn exports when concurrent writes happen (edge case noted in the spec) without requiring a database. Bundling audit snippets helps users trust exports without rerunning commands.
- **Alternatives considered**: (a) Rely on filesystem timestamps only - cannot guarantee determinism; (b) regenerate by diffing git history - unreliable for users who do not track Bases in git and violates local-first independence; (c) skip locking and ask users to retry manually - risks corrupted ZIPs highlighted in the spec guardrail.
