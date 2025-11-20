# Contracts - Profile Commands & APIs

Syntax expressed in pseudo-OpenAPI for Rust command handlers invoked through chat orchestrator.

---

## Command: `profile show <type>`

- **Request**
  ```json
  {
    "command": "profile_show",
    "profile_type": "writing",
    "base_id": "default-base",
    "render_options": { "include_history": true }
  }
  ```
- **Response**
  ```json
  {
    "profile_type": "writing",
    "metadata": { "id": "writing-profile", "last_updated": "2025-11-19T18:02:04Z", "scope": "this_base" },
    "summary": [
      "Uses confident, evidence-backed tone",
      "Prefers LaTeX math blocks for equations"
    ],
    "fields": { ... },
    "history_pointer": {
      "latest_event_id": "b6c0b246-...",
      "audit_command": "profile audit writing"
    },
    "attachments": [
      {
        "type": "html",
        "path": "/User/Base/profiles/writing.html",
        "hash": "c3ab8ff1..."
      }
    ]
  }
  ```
- **Errors**
  - `PROFILE_NOT_FOUND` (no JSON yet) -> instruct user to run interview.
  - `SCOPE_DISABLED` when metadata.scope = disabled.

---

## Command: `profile update <profile_type> <field>=<value>`

- **Request**
  ```json
  {
    "command": "profile_update",
    "profile_type": "work",
    "changes": {
      "focus_statement": "Submit CHI draft",
      "active_projects": [{ "name": "CHI Paper", "status": "drafting", "target_date": "2025-12-01" }]
    },
    "base_id": "default-base",
    "confirm": true
  }
  ```
- **Response**
  ```json
  {
    "status": "updated",
    "profile_type": "work",
    "event_id": "1337-...",
    "hash_after": "7719f0...",
    "diff_summary": [
      "Updated focus_statement",
      "Overwrote active_projects (1 entry)"
    ],
    "undo_token": "undo://profile/work/1337-..."
  }
  ```
- **Errors**
  - `VALIDATION_FAILED` (e.g., missing mastery_level).
  - `CONCURRENT_WRITE` if lock held; instruct user to retry.

---

## Command: `profile interview <type>`

- **Request**
  ```json
  {
    "command": "profile_interview",
    "profile_type": "knowledge",
    "base_id": "default-base",
    "mode": "guided",
    "requires_remote": true,
    "remote_prompt": {
      "operation_type": "profile_interview_knowledge",
      "data_categories": ["paper_excerpt"],
      "provider": "gpt-4o"
    }
  }
  ```
- **Flow**
  1. System generates manifest, presents consent summary.
  2. On approval, manifest stored and remote call executed.
  3. User reviews draft; accepts or edits before write.
- **Response**
  ```json
  {
    "status": "completed",
    "profile_type": "knowledge",
    "event_id": "...",
    "manifest_id": "...",
    "entries_added": 3,
    "entries_updated": 2
  }
  ```
- **Errors**
  - `CONSENT_DENIED` - store partial answers flagged `NEEDS_REMOTE_APPROVAL`.
  - `REMOTE_FAILURE` - log event with status `failed`, return actionable guidance.

---

## Command: `profile scope <type> <mode>`

- **Request**
  ```json
  {
    "command": "profile_scope",
    "profile_type": "writing",
    "scope_mode": "shared",
    "allowed_bases": ["default-base", "paper-b"]
  }
  ```
- **Response**
  ```json
  {
    "status": "updated",
    "profile_type": "writing",
    "scope_mode": "shared",
    "allowed_bases": ["default-base", "paper-b"],
    "event_id": "..."
  }
  ```
- **Constraints**: `shared` mode requires explicit allowed list; `disabled` prevents other APIs from exposing data.

---

## Command: `profile audit <type>`

- **Request**
  ```json
  {
    "command": "profile_audit",
    "profile_type": "work",
    "limit": 20
  }
  ```
- **Response**
  ```json
  {
    "profile_type": "work",
    "events": [
      {
        "event_id": "e1",
        "timestamp": "2025-11-10T17:34:11Z",
        "change_kind": "interview",
        "actor": "user",
        "diff_summary": ["Added project: NSF Grant"],
        "undo_token": "undo://profile/work/e1",
        "consent_manifest_ids": [],
        "hash_after": "..."
      }
    ],
    "undo_instructions": "Run profile undo work e1 to revert"
  }
  ```

---

## Command: `profile export <type>`

- **Request**
  ```json
  {
    "command": "profile_export",
    "profile_type": "knowledge",
    "destination": "/User/Base/profiles/exports"
  }
  ```
- **Response**
  ```json
  {
    "status": "exported",
    "profile_type": "knowledge",
    "archive_path": "/User/Base/profiles/exports/knowledge-2025-11-19T20-01-00Z.zip",
    "hash": "c2ab...",
    "event_id": "..."
  }
  ```
- **Errors**: `EXPORT_IN_PROGRESS`, `FILE_IO_ERROR`.

---

## Command: `profile delete <type>`

- **Request**
  ```json
  {
    "command": "profile_delete",
    "profile_type": "knowledge",
    "confirm_phrase": "DELETE knowledge"
  }
  ```
- **Response**
  ```json
  {
    "status": "deleted",
    "profile_type": "knowledge",
    "files_removed": [
      "/AI/Base/profiles/knowledge.json",
      "/User/Base/profiles/knowledge.html"
    ],
    "event_id": "..."
  }
  ```
- **Safeguards**: requires explicit confirm phrase; tells user how to run regenerate.

---

## Command: `profile regenerate --from-history <type>`

- **Request**
  ```json
  {
    "command": "profile_regenerate",
    "profile_type": "writing",
    "source": "history",
    "base_id": "default-base"
  }
  ```
- **Response**
  ```json
  {
    "status": "regenerated",
    "profile_type": "writing",
    "replayed_events": 12,
    "final_hash": "ab82...",
    "matches_last_known": true,
    "event_id": "..."
  }
  ```
- **Errors**: `MISSING_HISTORY`, `HASH_MISMATCH` (returns guidance to recover backups).

---

## Internal API: `profile.get_work_context()`

- **Signature**: `WorkContext profile::get_work_context(BaseId, ScopeCheck) -> Result<WorkContext, ProfileError>`
- **Output**
  ```json
  {
    "active_projects": [...],
    "focus_statement": "...",
    "milestones": [...],
    "scope_mode": "this_base",
    "last_updated": "..."
  }
  ```
- **Notes**: Returns filtered view depending on scope; errors if disabled or profile missing.

---

## Internal API: `profile.get_knowledge_summary()`

- **Signature**: `KnowledgeSummary profile::get_knowledge_summary(BaseId) -> Result<KnowledgeSummary, ProfileError>`
- **Output**
  ```json
  {
    "counts": {
      "strengths": 4,
      "weaknesses": 2,
      "unverified": 1
    },
    "entries": [
      {
        "concept": "Graph Neural Networks",
        "mastery_level": "developing",
        "evidence_refs": ["paper:citation-id"],
        "weakness_flags": ["Needs more ablation knowledge"]
      }
    ],
    "stale_evidence_refs": [
      { "concept": "Bayesian EU", "missing_reference": "notes/123.md" }
    ]
  }
  ```
- **Notes**: Exposed to Learning Mode; respects scope and flags pending remote approvals.

---

## Error Codes Summary

| Code | Meaning | User Guidance |
|------|---------|---------------|
| `PROFILE_NOT_FOUND` | Profile JSON missing | Run `profile interview <type>` |
| `SCOPE_DISABLED` | User disabled sharing | Re-enable via `profile scope` |
| `VALIDATION_FAILED` | Input failed schema rules | Surface specific field errors |
| `CONCURRENT_WRITE` | Lock held | Retry after current write completes |
| `CONSENT_DENIED` | User denied manifest | Entry stored as `NEEDS_REMOTE_APPROVAL` |
| `REMOTE_FAILURE` | Upstream provider error | Log event, prompt to retry or continue locally |
| `EXPORT_IN_PROGRESS` | Another export running | Wait or cancel |
| `MISSING_HISTORY` | Orchestration logs unavailable | Provide recovery instructions |
| `HASH_MISMATCH` | Regenerated state diverged | Stop, alert user, link to audit |
