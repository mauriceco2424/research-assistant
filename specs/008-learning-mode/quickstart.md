# Quickstart: Learning Mode Sessions

**Branch**: 008-learning-mode  
**Date**: 2025-11-21  
**Spec**: specs/008-learning-mode/spec.md

## Goal
Enable chat-first learning sessions (Quiz/Oral Exam) scoped to Base/categories/papers/concepts, with KnowledgeProfile-driven questions, feedback, logged updates, undo, and regenerability.

## Steps

1. **Start session (chat command)**
   - Prompt includes scope (Base/categories/papers/concepts) and mode (Quiz or Oral Exam).
   - System confirms scope/mode, states local-only operation, and initializes session context with a default of 5 questions.
   - Example: `/learning start mode=quiz scope=categories:ml,rl`

2. **Run Q&A loop**
   - System generates questions from KnowledgeProfile gaps for the scope.
   - User answers in chat; system evaluates, returns corrective feedback, and suggests follow-ups.
   - User can continue after the default 5 or stop early in chat.

3. **Apply updates**
   - After each evaluation, update KnowledgeProfile coverage/competence/difficulty; log orchestration events for generation, evaluation, updates, and undo.
   - Ensure artifacts (prompts, responses, evaluations) are stored locally for regeneration.
   - Sample reply: `continue` to keep asking after 5, or `stop` to end.

4. **Summarize and undo**
   - Provide session summary (questions, outcomes, KnowledgeProfile changes, recommendations).
   - Allow undo of the latest KnowledgeProfile update via chat; log the reversal.
   - Example: `/learning summary {session_id}` then `/learning undo {session_id}`

## Compliance reminders
- **P1/P2**: No hidden network calls; any external model use must request explicit approval per batch.
- **P3/P4**: Store AI-layer artifacts so sessions are regenerable from disk plus Base contents.
- **P5**: All controls stay in chat; no new complex UI.
- **P6**: Log orchestration events and support undo of latest KnowledgeProfile update.
- **P8**: Ensure learning records capture sessions, questions, answers, and summaries.
