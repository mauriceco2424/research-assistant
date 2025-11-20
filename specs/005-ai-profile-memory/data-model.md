# Data Model - AI Profiles & Long-Term Memory

## Storage Layout

- **AI Layer**: `/AI/<Base>/profiles/{user|work|writing|knowledge}.json`
- **User Layer Summaries**: `/User/<Base>/profiles/{user|work|writing|knowledge}.html`
- **Consent Manifests**: `/AI/<Base>/consent/manifests/{manifest_id}.json`
- **Exports**: `/User/<Base>/profiles/exports/{profile_type}-{timestamp}.zip`
- **Audit Logs**: Existing orchestration log under `/AI/<Base>/orchestration/events.jsonl` gains profile-specific payloads.

All JSON files serialize with deterministic key ordering (`metadata`, `summary`, `fields`, `history`).

## Entities

### UserProfile

- **Purpose**: Captures researcher identity, collaboration norms, communication preferences.
- **Fields**:
  | Field | Type | Rules |
  |-------|------|-------|
  | `metadata.id` | UUID | Stable per Base (`user-profile`) |
  | `metadata.last_updated` | ISO-8601 string | Updated on every mutation |
  | `metadata.scope` | enum {`this_base`, `shared`, `disabled`} | Controls downstream usage |
  | `summary` | array[string] | Pre-rendered highlights shown in chat |
  | `fields.name` | string | Required; derived from onboarding |
  | `fields.affiliations` | array[string] | Optional |
  | `fields.communication_style` | array[string] | Strings such as "direct", "concise" |
  | `fields.availability` | string | Freeform but validated against allowed windows |
  | `history` | array[`HistoryRef`] | Reverse chronological pointer to orchestration events |

### WorkProfile

- **Purpose**: Tracks active projects, deadlines, TODO themes.
- **Fields**:
  | Field | Type | Rules |
  |-------|------|-------|
  | `metadata` | same shape as UserProfile (id = `work-profile`) |
  | `fields.active_projects` | array[ProjectRef] | Each project has `name`, `status`, `target_date` |
  | `fields.milestones` | array[Milestone] | `description`, `due`, `evidence_refs[]` |
  | `fields.preferred_tools` | array[string] | Optional |
  | `fields.focus_statement` | string | e.g., "Submit CHI draft" |
  | `fields.risks` | array[string] | Highlights schedule blockers |

### WritingProfile

- **Purpose**: Encodes tone, formatting, citation expectations, exemplar snippets.
- **Fields**:
  | Field | Type | Rules |
  |-------|------|-------|
  | `metadata` | same as others (id = `writing-profile`) |
  | `fields.tone_descriptors` | ordered array[string] | Required; each entry <= 50 characters |
  | `fields.structure_preferences` | array[string] | e.g., "IMRAD", "bullets + summary" |
  | `fields.style_examples` | array[ExampleRef] | Each holds `source`, `excerpt`, `citation` |
  | `fields.remote_inference_metadata` | object | Contains `last_remote_source`, `consent_manifest_id`, `status` {`approved`, `rejected`, `pending`} |

### KnowledgeProfile

- **Purpose**: Maintains concept mastery for Learning Mode.
- **Fields**:
  | Field | Type | Rules |
  |-------|------|-------|
  | `metadata` | same pattern (id = `knowledge-profile`) |
  | `entries` | array[KnowledgeEntry] | At least empty array present |
  | `summary` | optional derived bullets | Rendered fields like "3 strengths, 2 weaknesses" |
  | `history` | array[`HistoryRef`] | same as other profiles |

#### KnowledgeEntry

- **Fields**:
  | Field | Type | Rules |
  |-------|------|-------|
  | `concept` | string | Required unique key |
  | `mastery_level` | enum {`novice`,`developing`,`proficient`,`expert`} | Defaults to `developing` |
  | `evidence_refs` | array[EvidenceRef] | Each ref includes `type` (`paper`,`note`,`manual`), `id/path`, `confidence` (0-1). Must contain >= 1 entry or mark `verification_status = "UNVERIFIED"`. |
  | `weakness_flags` | array[string] | e.g., "lacks proof intuition" |
  | `learning_links` | array[LearningLink] | Pointers to Learning Mode tasks |
  | `last_reviewed` | ISO-8601 string | Required |
  | `verification_status` | enum {`VERIFIED`,`UNVERIFIED`,`STALE`} | Derived from evidence freshness |

### ProfileChangeEvent

- Stored in orchestration log.
- **Fields**:
  | Field | Type | Rules |
  |-------|------|-------|
  | `event_id` | UUID | Primary key |
  | `profile_type` | enum {`user`,`work`,`writing`,`knowledge`} | Required |
  | `change_kind` | enum {`create`,`interview`,`manual_edit`,`scope_change`,`export`,`delete`,`regenerate`} |
  | `timestamp` | ISO-8601 string | Generated server-side |
  | `actor` | string | Typically `user` vs `ai_orchestrator` |
  | `diff_summary` | array[string] | Human-readable explanation of field-level diffs |
  | `undo_token` | string | Opaque token referencing rollback instructions |
  | `consent_manifest_ids` | array[UUID] | Optional; ties to remote approvals |
  | `hash_before` / `hash_after` | hex string | SHA-256 of profile JSON before/after change |

### ConsentManifest

- **Fields**:
  | Field | Type | Rules |
  |-------|------|-------|
  | `manifest_id` | UUID | File name |
  | `operation_type` | string | e.g., `profile_interview_writing` |
  | `data_categories` | array[string] | "metadata_only", "paper_excerpt" |
  | `provider` | string | Remote endpoint identifier |
  | `prompt_excerpt` | string | Sanitized snippet |
  | `approved_at` | ISO-8601 string | Required |
  | `expires_at` | ISO string | Optional |
  | `profiles_touched` | array[profile_type] | Which profiles may use the result |
  | `status` | enum {`approved`,`rejected`,`revoked`} |

### ProfileScopeSetting

- Represented either as part of metadata or a shared config file.
- **Fields**:
  | Field | Type | Rules |
  |-------|------|-------|
  | `profile_type` | enum | Key |
  | `scope_mode` | enum {`this_base`,`shared`,`disabled`} |
  | `allowed_bases` | array[string] | Only used when shared |
  | `updated_at` | ISO-8601 string | |

### HTML Summary Artifact

- Generated from JSON; contains:
  | Field | Type | Rules |
  |-------|------|-------|
  | `profile_type` | string | Lowercase |
  | `rendered_at` | ISO-8601 string | |
  | `source_hash` | hex string | Must match JSON hash |

## Relationships & Constraints

- **Profiles <-> Orchestration Events**: Every mutation appends a `HistoryRef` (`{event_id, timestamp, hash_after}`) to the profile JSON; `profile audit` cross-references events for detailed logs.
- **Knowledge Entries <-> Evidence**: `EvidenceRef.id` links to either `/User/<Base>/papers/<id>` or `/AI/<Base>/notes/<id>.md`. If target moved/deleted, regeneration marks entry `verification_status = STALE`.
- **Consent Manifests <-> Interviews**: Interview flows record manifest ID in both the profile metadata (`remote_inference_metadata.last_manifest_id`) and the corresponding orchestration event.
- **Scope Settings <-> Orchestrations**: Orchestrator checks metadata.scope before injecting profile data into other commands; disabled scope prevents even read-only API hooks from returning values.

## State Transitions

1. **Interview Initiated** -> `status = pending`, collects answers, optionally requests remote inference.
2. **User Confirms** -> `status = confirmed`; profile JSON patched; `last_updated` + `history` updated.
3. **User Cancels** -> `status = canceled`; no write occurs but event logs the cancellation for transparency.
4. **Delete Requested** -> profile JSON + HTML removed; event recorded; `profile regenerate` can recreate using history.
5. **Regenerate** -> Replays `history` events until latest hash matches recorded `hash_after`; if mismatch, state set to `needs_attention`.

## Validation Rules

- All timestamps stored in UTC ISO-8601 with timezone (`Z`).
- Scope mode `shared` requires explicit list of Base IDs; empty list defaults to disabled.
- Hash comparisons performed before overwriting files; mismatch aborts write.
- Evidence references referencing missing files mark entry as `STALE` but do not delete original data.

## Derived APIs

- `profile.get_work_context()` returns subset of WorkProfile fields filtered by scope.
- `profile.get_knowledge_summary()` returns aggregated counts (strengths, weaknesses, unverified entries) and list of entries flagged `weakness_flags` not empty.
- `profile.list_pending_remote()` surfaces writing profile entries whose `remote_inference_metadata.status = pending`.
