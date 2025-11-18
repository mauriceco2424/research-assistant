# Data Model – Onboarding & Paper Acquisition Workflow

**Feature**: `001-onboarding-paper-acquisition`  
**Spec**: `specs/001-onboarding-paper-acquisition/spec.md`  
**Date**: 2025-11-18

This document describes the key entities, fields, and relationships required
to implement the onboarding and paper acquisition workflows while respecting
the ResearchBase constitution.

---

## 1. Paper Base

Represents a distinct research domain with its own library and AI-layer
memory.

- **Fields (conceptual)**:
  - `id` – stable identifier for the Base.
  - `name` – human-readable Base name.
  - `user_layer_path` – filesystem path to the User Layer directory.
  - `ai_layer_path` – filesystem path to the AI Layer directory.
  - `created_at` – creation timestamp.
  - `last_active_at` – last time this Base was active.
  - `settings` – Base-specific configuration (e.g., acquisition preferences,
    enabled services).
- **Relationships**:
  - Has many **Library Entries**.
  - Has many **Acquisition Batches**.
  - Has many **Orchestration Events**.

**Validation / Constraints**:

- `user_layer_path` and `ai_layer_path` MUST be distinct and reside on the
  local filesystem.
- For any active Base, both directories MUST exist or be created on demand.

---

## 2. Paper Candidate

Represents a suggested paper that has not yet been fully integrated into the
library.

- **Fields**:
  - `candidate_id` – ephemeral identifier within an acquisition operation.
  - `title`
  - `authors` – list of strings.
  - `venue` – journal/conference or source.
  - `year` – publication year, if known.
  - `identifier` – primary stable ID (e.g., DOI, arXiv ID, URL).
  - `source` – where the candidate came from (e.g., "PathBInterview",
    "DiscoverySearch").
- **Relationships**:
  - Belongs to an **Acquisition Batch** when selected for acquisition.

**Validation / Constraints**:

- `identifier` MUST be present before acquisition is attempted.
- Candidates are not persisted long-term as-is; once acquired, they become
  Library Entries or remain only in orchestration history.

---

## 3. Library Entry

Represents a paper that is part of a Base's library.

- **Fields**:
  - `entry_id` – stable identifier within the Base.
  - `base_id` – reference to owning Paper Base.
  - `title`
  - `authors`
  - `venue`
  - `year`
  - `identifier` – DOI/ID/URL.
  - `pdf_paths` – list of filesystem paths to attached PDFs (User Layer).
  - `needs_pdf` – boolean flag indicating that no accessible PDF is attached.
  - `created_at`
  - `updated_at`
- **Relationships**:
  - Part of exactly one **Paper Base**.
  - May be referenced by categories, reports, and learning sessions (outside
    the scope of this feature).

**Validation / Constraints**:

- `needs_pdf` MUST be true if and only if `pdf_paths` is empty.
- When acquisition attaches a PDF, `pdf_paths` MUST be updated and
  `needs_pdf` set to false.

---

## 4. Acquisition Batch

Represents a single user-approved acquisition operation.

- **Fields**:
  - `batch_id` – unique identifier.
  - `base_id` – reference to owning Paper Base.
  - `requested_at` – timestamp when the AI proposed candidates.
  - `approved_at` – timestamp when the user approved acquisition.
  - `approved_by` – descriptor of the approval source (e.g., chat message ID).
  - `candidates` – list of candidate descriptors (title, identifier, etc.).
  - `results` – per-candidate outcome:
    - `metadata_success` (bool)
    - `pdf_success` (bool)
    - `library_entry_id` (if created)
    - `error` (if any).
- **Relationships**:
  - Belongs to a **Paper Base**.
  - May be linked from one or more **Orchestration Events**.

**Validation / Constraints**:

- An Acquisition Batch MUST only be created after explicit user approval.
- Batches MUST be immutable after initial recording, except for adding
  audit-only fields (e.g., annotations).

---

## 5. Orchestration Event

Generic record of AI-orchestrated operations, including but not limited to
acquisition.

- **Fields**:
  - `event_id` – unique identifier.
  - `base_id` – reference to owning Paper Base.
  - `event_type` – e.g., "paper_acquisition", "category_generation".
  - `timestamp`
  - `initiator` – user/chat context initiating the operation.
  - `payload` – structured details of the operation (JSON/Markdown).
  - `related_batch_id` – optional link to an Acquisition Batch.
- **Relationships**:
  - Belongs to a **Paper Base**.

**Validation / Constraints**:

- Every acquisition MUST have at least one corresponding Orchestration Event.
- Orchestration Events MUST be stored in AI Layer in a diff-friendly format.

---

## 6. Base Selection and Onboarding State

This feature assumes minimal additional state for onboarding:

- **Fields (conceptual)**:
  - `last_active_base_id` – stored in a global configuration to preselect Base
    on startup.
  - `onboarding_status` per Base – simple markers such as "not_started",
    "path_a_completed", "path_b_completed".

**Validation / Constraints**:

- Lack of onboarding status MUST NOT prevent the Base from being used; it is
  purely informative.

