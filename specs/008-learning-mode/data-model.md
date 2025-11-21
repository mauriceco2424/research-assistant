# Data Model: Learning Mode Sessions

**Branch**: 008-learning-mode  
**Date**: 2025-11-21  
**Spec**: specs/008-learning-mode/spec.md

## Entities

### LearningSession
- **id**: unique session identifier.
- **scope**: one of `base`, `categories[]`, `papers[]`, `concepts[]`.
- **mode**: `quiz` | `oral_exam`.
- **status**: `pending` | `active` | `completed` | `cancelled`.
- **question_default_count**: integer (default 5).
- **start_timestamp** / **end_timestamp**: UTC.
- **events_ref**: references to orchestration event log entries.
- **regeneration_pointer**: reference to AI-layer materials required to replay session (prompt manifests, seeds, scope selection).

### QuestionItem
- **id**: unique question id per session.
- **session_id**: foreign key to LearningSession.
- **prompt**: question text.
- **target_concepts**: list of concept ids or names linked to KnowledgeProfile.
- **target_papers**: list of paper ids (if applicable).
- **difficulty**: enum or numeric difficulty tier at generation time.
- **selection_rationale**: text describing why chosen (gap, weak area).
- **expected_answer_outline**: structured outline for evaluation.

### ResponseEvaluation
- **id**: unique evaluation id.
- **question_id**: foreign key to QuestionItem.
- **user_answer**: captured answer text.
- **evaluation_outcome**: `correct` | `partial` | `incorrect`.
- **feedback**: corrective feedback text.
- **follow_up_recommendations**: list of suggested actions (papers, concepts, exercises).
- **kp_update_ref**: reference to KnowledgeProfile updates applied from this evaluation.

### KnowledgeProfile Entry
- **id**: concept/paper/category entry id.
- **coverage**: numeric or categorical coverage level.
- **competence**: numeric or categorical competence level.
- **difficulty**: difficulty signal for the material.
- **update_history**: ordered list of updates (session/question ids, timestamps, deltas).
- **undo_markers**: references to reversible updates (last N).

## Relationships
- LearningSession 1..* QuestionItem.
- QuestionItem 0..1 ResponseEvaluation (per question per attempt; single evaluation assumed per question for this feature).
- ResponseEvaluation 1..* KnowledgeProfile updates (captured via kp_update_ref).

## Validation Rules & Constraints
- question_default_count defaults to 5; user prompts in chat can extend or stop the session.
- A session cannot transition to `completed` without at least one evaluated question or an explicit empty-scope confirmation.
- Undo operations only target the latest KnowledgeProfile update per session (stack behavior).
- All artifacts and logs are stored locally under the Base to respect P1/P2; external calls require explicit approval per batch before generation/evaluation.

## State Transitions (LearningSession)
- `pending` → `active` (after scope/mode confirmation).
- `active` → `completed` (after finishing question set or user stops).
- `active` → `cancelled` (user cancels mid-session; partial progress logged).
- `completed`/`cancelled` → (optional) `active` if session is resumed with remaining or new questions; must log resumption.
