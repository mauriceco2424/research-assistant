# Feature Specification: AI Profiles & Long-Term Memory

**Feature Branch**: `005-ai-profile-memory`  
**Created**: 2025-11-19  
**Status**: Draft  
**Input**: User description: "Design the AI Profiles capability described in master_spec.md Section 7 and roadmap item Spec 05. Produce a spec that defines how the system captures, stores, surfaces, and updates the four long-term profiles (UserProfile, WorkProfile, WritingProfile, KnowledgeProfile) entirely within the AI layer, aligning with constitutional principles (P1-P10) and building on the completed Specs 01-04."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Profile inspection & inline edits (Priority: P1)

After onboarding, the researcher can ask `profile show <user|work|writing|knowledge>` inside chat to see a structured summary (JSON and rendered bullets) and then issue `profile update <field>` commands to tweak tone, goals, or work context without touching files.

**Why this priority**: Trustworthy visibility into stored preferences is foundational; without transparent summaries and edits the remaining profile work has no anchor.

**Independent Test**: From a fresh Base with seeded profile JSON, run chat commands to show and edit each profile and confirm summaries, field-level updates, timestamps, and event IDs appear without involving other flows.

**Acceptance Scenarios**:

1. **Given** a Base that already captured onboarding data, **When** the user runs `profile show writing`, **Then** the assistant returns structured tone guidance with last-modified timestamp, evidence references, and a pointer to audit history.
2. **Given** a researcher reviewing their work focus, **When** they run `profile update work focus="Submit CHI draft"`, **Then** the system validates the change, confirms the diff, writes the JSON artifact locally, and reports the new orchestration event ID.

---

### User Story 2 - Guided interviews & capture runs (Priority: P2)

The researcher can launch guided interviews such as `profile interview knowledge` or `profile run writing-style` that gather missing data, request consent for any remote inference, and confirm before overwriting stored values.

**Why this priority**: Interviews keep long-term memories current and reduce repeated questioning, unlocking contextualized assistance without manual editing.

**Independent Test**: Trigger each interview mode separately, mock both local-only prompts and ones that require remote summarization, and verify that consent manifests, confirmation prompts, and storage happen even if no other commands run.

**Acceptance Scenarios**:

1. **Given** the user starts `profile interview knowledge`, **When** the flow detects remote tone extraction from a PDF, **Then** it pauses, presents the prompt manifest, records the approval, and only then stores the resulting data with consent metadata.
2. **Given** the writing profile already holds tone preferences, **When** the user reruns `profile run writing-style`, **Then** the system lets them accept, edit, or reject the new draft before any JSON overwrite occurs.

---

### User Story 3 - Knowledge readiness for Learning Mode (Priority: P3)

Before entering Learning Mode, the researcher reviews the KnowledgeProfile inside chat, tags strengths or weaknesses, links entries to papers/notes, and ensures mastery levels match evidence.

**Why this priority**: Learning Mode depends on accurate knowledge snapshots; users need a dedicated flow to curate mastery statements and references without relying on future features.

**Independent Test**: Populate a Base with several knowledge entries, issue `profile show knowledge` and mark adjustments, and verify Learning Mode integrations (e.g., `profile.get_knowledge_summary`) deliver consumable JSON even without other profile work.

**Acceptance Scenarios**:

1. **Given** a knowledge entry referencing a paper, **When** the user marks it as "needs review," **Then** the system logs the weakness flag, retains the evidence citation, and exposes the change via API/hooks for Learning Mode to consume.
2. **Given** the user links a concept to an existing note, **When** the note is moved or deleted, **Then** the knowledge profile marks the evidence as stale and surfaces it the next time the user inspects the profile.

---

### User Story 4 - Profile governance, audit, and regenerability (Priority: P4)

Researchers invoke `profile audit <type>`, `profile export`, `profile delete <type>`, or `profile regenerate --from-history` to inspect change history, export ZIPs to /User/<Base>/profiles/, wipe a profile, or rebuild artifacts deterministically.

**Why this priority**: Governance tools enforce constitutional guarantees (local-first storage, transparency, regenerability) and make long-term memory trustworthy.

**Independent Test**: Without running interviews, execute each governance command independently to confirm exports, deletions, and regenerations work, respect consent logs, and provide undo guidance.

**Acceptance Scenarios**:

1. **Given** multiple profile edits occurred across sessions, **When** the user runs `profile audit work`, **Then** the assistant lists chronological orchestration events with who/what/when details plus undo instructions per entry.
2. **Given** a user requests `profile delete knowledge`, **When** the command executes, **Then** the KnowledgeProfile JSON and HTML summary are removed locally, an event log entry is created, and the user receives instructions on how to regenerate from history if needed.

---

### Edge Cases

- Profile show is requested before any onboarding data exists -> fall back to scaffolded defaults, mark fields as "uninitialized," and prompt the user to run an interview instead of returning errors.
- Remote inference is rejected by the user mid-interview -> store a partial entry tagged as `NEEDS_REMOTE_APPROVAL` so future flows know why data is incomplete.
- Regeneration is attempted after orchestration logs were pruned or missing -> alert the user, block regeneration, and instruct them to recover backups instead of silently producing divergent JSON.
- Profile export is invoked while another write is in progress -> queue or retry after the active write completes to avoid torn archives, and record both attempts in the audit log.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST persist one JSON artifact per profile under `/AI/<Base>/profiles/<profile>.json` plus optional human-readable HTML summaries under `/User/<Base>/profiles/`, ensuring only local filesystem writes (P1).
- **FR-002**: System MUST expose chat commands `profile show`, `profile update`, `profile scope`, `profile export`, `profile delete`, `profile interview`, and `profile audit` that can target UserProfile, WorkProfile, WritingProfile, or KnowledgeProfile without requiring file access.
- **FR-003**: Chat responses to `profile show` MUST include structured summaries (key-value, timestamps, evidence references) and a pointer to the latest orchestration event ID for that profile.
- **FR-004**: Profile edits initiated through chat MUST run a confirmation step showing differences, capture the user's acknowledgement, and log the change as an orchestration event with undo guidance (P6).
- **FR-005**: Guided interviews MUST track completion state, allow cancel/retry, and record whether values came from user input, local inference, or remote inference (with consent manifests when external endpoints are used per P2).
- **FR-006**: KnowledgeProfile entries MUST store `concept`, `mastery_level`, `evidence_refs` (paper IDs, note IDs, or manual citations), `weakness_flags`, and optional links to Learning Mode tasks; entries missing evidence MUST be labeled as unverified per P7.
- **FR-007**: Profile scope controls MUST let the user declare where each profile can be applied (current Base only, shared across Bases, or temporarily disabled) and enforce those settings in downstream orchestrations.
- **FR-008**: Export and delete commands MUST operate locally (ZIP to `/User/<Base>/profiles/`, delete JSON/HTML artifacts), respect per-profile selections, and confirm completion in chat to satisfy privacy/undo expectations (P1, P6).
- **FR-009**: `profile audit <type>` MUST gather chronological orchestration events, include actor/context references, surface pending undo tokens, and highlight remote consent references so users understand why changes occurred.
- **FR-010**: `profile regenerate --from-history <type>` MUST replay orchestration logs deterministically to rebuild the JSON artifact; on mismatched hashes it MUST alert the user and stop to preserve regenerability guarantees (P3/P4).
- **FR-011**: API hooks (e.g., `profile.get_work_context()`, `profile.get_knowledge_summary()`) MUST expose read-only snapshots for other specs (Learning Mode, planning) without bypassing scope or consent rules.
- **FR-012**: The system MUST initialize existing Bases (Specs 01-04) with default empty profile shells plus migration notes so legacy users gain immediate visibility and are prompted to populate missing data.

### Key Entities *(include if feature involves data)*

- **UserProfile**: Captures researcher identity, background, collaboration preferences, and communication tone; fields include `name`, `affiliations`, `communication_style`, `availability`, `scope_flags`, `last_updated`.
- **WorkProfile**: Describes current projects, deadlines, TODO themes, and Base-specific constraints; fields include `active_projects`, `milestones`, `preferred_tools`, `scope_flags`, and references to orchestration events or tasks.
- **WritingProfile**: Stores voice characteristics, formatting expectations (LaTeX, citation styles), and exemplar snippets; includes `tone_descriptors`, `structure_preferences`, `style_examples`, `remote_inference_metadata`.
- **KnowledgeProfile**: Maintains mastery records keyed by concept; attributes include `concept`, `mastery_level`, `evidence_refs`, `weakness_flags`, `learning_links`, and `last_reviewed`.
- **ProfileChangeEvent**: Logical log entry for every mutation, containing `event_id`, `profile_type`, `change_summary`, `actor`, `timestamp`, `undo_instructions`, and `consent_manifest_ids` when applicable.
- **ConsentManifest**: References remote inference requests with `endpoint_purpose`, `prompt_excerpt`, `approved_at`, `expiry`, and `profiles_touched`, ensuring P2 compliance.

## Assumptions & Dependencies

- Existing onboarding flows (Specs 01-04) already capture seed data that can populate initial profile shells; this spec only defines how to reuse or expand that data.
- The chat router can dispatch `profile *` commands without new UI panes, and HTML summaries remain optional attachments triggered from chat (P5).
- Orchestration logging infrastructure already exists and can be extended with profile-specific event types and undo tokens.
- Remote inference, when required, uses previously approved providers; if no provider is configured the interview gracefully skips remote steps and logs the omission.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 95% of `profile show <type>` commands return structured summaries (JSON + narrative) within 5 seconds and include timestamps plus audit references, enabling users to inspect any profile entirely from chat.
- **SC-002**: At least 90% of guided interview sessions complete without manual JSON editing, and every overwrite action includes an explicit confirmation step logged with an orchestration event ID.
- **SC-003**: 100% of KnowledgeProfile entries contain at least one evidence reference or are flagged as `UNVERIFIED`, and Learning Mode integrations can consume the exported JSON without additional normalization work.
- **SC-004**: When testers run `profile export`, `profile delete`, and `profile regenerate --from-history` back-to-back, regenerated JSON hashes match the pre-export state in 100% of cases (barring missing logs), demonstrating regenerability and auditability.
