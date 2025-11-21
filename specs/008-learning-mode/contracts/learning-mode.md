# Contracts: Learning Mode Sessions (Chat-Orchestrated)

**Branch**: 008-learning-mode  
**Date**: 2025-11-21  
**Interface Surface**: Chat commands mediated by orchestrator; modeled as logical endpoints for clarity.

## Start Session
- **Identifier**: `learning.start`
- **Request**
  - `scope`: `base` | `categories[]` | `papers[]` | `concepts[]`
  - `mode`: `quiz` | `oral_exam`
  - `question_default_count`: optional integer (defaults to 5)
- **Response**
  - `session_id`
  - `confirmed_scope`, `mode`, `question_default_count`
  - `message`: confirmation string
  - `events_ref`: orchestration event id for session start
- **Errors**
  - `invalid_scope` (empty KnowledgeProfile segment)
  - `remote_disallowed` (if external model requested without approval)

## Next Question
- **Identifier**: `learning.next_question`
- **Request**
  - `session_id`
- **Response**
  - `question_id`
  - `prompt`
  - `target_concepts`
  - `target_papers`
  - `difficulty`
  - `selection_rationale`
- **Errors**
  - `session_not_active`
  - `question_limit_reached` (after default 5) with guidance to continue/stop

## Submit Answer
- **Identifier**: `learning.submit_answer`
- **Request**
  - `session_id`
  - `question_id`
  - `user_answer`
- **Response**
  - `evaluation_outcome`: `correct` | `partial` | `incorrect`
  - `feedback`
  - `follow_up_recommendations`
  - `kp_update_ref`
  - `events_ref`: orchestration event id for evaluation and KnowledgeProfile update
- **Errors**
  - `session_not_active`
  - `question_not_found`
  - `evaluation_failed` (explainable reason; no hidden network)

## Continue or Stop
- **Identifier**: `learning.control`
- **Request**
  - `session_id`
  - `action`: `continue` | `stop`
- **Response**
  - `status`: updated session status
  - `message`: confirmation
- **Errors**
  - `session_not_active`

## Undo Last Update
- **Identifier**: `learning.undo_last_update`
- **Request**
  - `session_id`
- **Response**
  - `undone_update_ref`
  - `events_ref`: orchestration event id for undo
- **Errors**
  - `no_updates_to_undo`
  - `session_not_found`

## Session Summary
- **Identifier**: `learning.summary`
- **Request**
  - `session_id`
- **Response**
  - `questions_overview`
  - `evaluation_outcomes`
  - `knowledge_profile_changes`
  - `recommendations`
  - `regeneration_pointer`
- **Errors**
  - `session_not_found`
  - `session_not_completed` (allow partial summary with warning)
