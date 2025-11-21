# Tasks: Learning Mode Sessions

**Feature**: 008-learning-mode  
**Spec**: specs/008-learning-mode/spec.md  
**Plan**: specs/008-learning-mode/plan.md  
**Branch**: 008-learning-mode  
**Date**: 2025-11-21

## Phase 1 - Setup

- [X] T001 Confirm repo context and branch `008-learning-mode` active; ensure feature docs present in `specs/008-learning-mode/` (spec, plan, research, data model, contracts, quickstart).
- [X] T002 Inventory current chat/orchestration/KnowledgeProfile modules in `src/chat`, `src/orchestration`, `src/profiles` to align extension points.

## Phase 2 - Foundational

- [X] T003 Define learning session domain module skeleton in `src/chat/learning_sessions.rs` (structs, enums, orchestration hooks) aligned to data-model entities.
- [X] T004 Add persistence hooks for session artifacts/logs under AI layer in `src/storage/ai_layer.rs` (local-only writes, regeneration pointers).
- [X] T005 Wire orchestration logging types needed for learning events in `src/orchestration/events.rs` (generation, evaluation, KP update, undo).
- [X] T006 Extend KnowledgeProfile update interface in `src/profiles/mod.rs` to accept session/question-scoped updates and undo markers.

## Phase 3 - User Story 1 (Start scoped session via chat) - Priority P1

**Goal**: Start a learning session from chat with scope/mode selection, local-only confirmation, default 5 questions, and session context stored.

- [X] T007 [US1] Implement chat command parser/handler for learning session start with scope/mode in `src/chat/handlers/learning_start.rs`.
- [X] T008 [US1] Add scope validation (Base/categories/papers/concepts) including empty-scope guardrail in `src/chat/learning_sessions.rs`.
- [X] T009 [US1] Persist session context (scope, mode, default question count, timestamps) in AI layer via `src/storage/ai_layer.rs`.
- [X] T010 [US1] Emit orchestration event for session start with prompt manifest/local-only notice in `src/orchestration/events.rs`.
- [X] T011 [US1] Provide chat confirmations and error responses (invalid scope, no content, network disallowed) in `src/chat/handlers/learning_start.rs`.

## Phase 4 - User Story 2 (Targeted Q&A with feedback) - Priority P2

**Goal**: Generate questions from KnowledgeProfile gaps, present in chat, evaluate answers with corrective feedback and recommendations, adapt difficulty, store artifacts.

- [X] T012 [US2] Implement question generation routine using KP gaps/difficulty in `src/chat/learning_sessions.rs` (respects scope, local-only unless approved).
- [X] T013 [US2] Add chat surface for presenting questions and capturing answers in `src/chat/handlers/learning_qna.rs`.
- [X] T014 [US2] Implement evaluation flow returning outcome, corrective feedback, and follow-up recs in `src/chat/learning_sessions.rs`.
- [X] T015 [US2] Adapt in-session difficulty selection based on prior answers/KP signals in `src/chat/learning_sessions.rs`.
- [X] T016 [US2] Persist question/answer/evaluation artifacts with rationales in AI layer via `src/storage/ai_layer.rs`.
- [X] T017 [US2] Log orchestration events for question generation and evaluation with scope/mode metadata in `src/orchestration/events.rs`.
- [X] T018 [US2] Ensure default run of 5 questions with chat controls to continue/stop in `src/chat/handlers/learning_qna.rs`.

## Phase 5 - User Story 3 (Review & undo KP updates) - Priority P3

**Goal**: Summarize session changes to KnowledgeProfile, and allow undo of latest update with logging.

- [X] T019 [US3] Generate session summary (questions, outcomes, KP deltas, recommendations) in `src/chat/handlers/learning_summary.rs`.
- [X] T020 [US3] Implement undo-latest KP update for session with stack behavior in `src/profiles/mod.rs`.
- [X] T021 [US3] Log undo orchestration event and maintain linkage to original update in `src/orchestration/events.rs`.
- [X] T022 [US3] Provide chat commands/responses for summary and undo (success/failure states) in `src/chat/handlers/learning_summary.rs`.

## Final Phase - Polish & Cross-Cutting

- [X] T023 [P] Add regeneration pointer handling and dry-run regeneration check for session artifacts in `src/storage/ai_layer.rs`.
- [X] T024 [P] Update `specs/008-learning-mode/quickstart.md` with any command syntax/examples finalized during implementation.
- [X] T025 [P] Add manual verification checklist for chat flows (start, 5-question loop, continue/stop, summary, undo) in `specs/008-learning-mode/tasks.md#Manual Verification` section.

## Dependencies (User Story Order)

- US1 (session start) → US2 (Q&A loop) → US3 (summary/undo).

## Parallel Execution Examples

- Run T003/T005/T006 in parallel (domain/logging/KP interface) before story work.
- Within US2, T012/T013/T015 can proceed in parallel once T007–T011 complete; T016/T017 depend on generation/evaluation wiring.
- Polish tasks T023–T025 can run after core story tasks complete.

## Implementation Strategy (MVP then iterate)

- MVP: Deliver US1 fully; then US2 with 5-question default and feedback; add US3 summary/undo to complete core loop.
- Iterate: Enhance difficulty tuning, extend regeneration checks, refine recommendations.

## Manual Verification

- Start session via chat with scope/mode; confirm local-only notice and default 5 questions.
- Complete 5-question loop; observe feedback and ability to continue/stop.
- Inspect AI-layer artifacts and orchestration logs for generation/evaluation events.
- Run summary; verify KP updates reported and undo reverses latest change with logged event.
- Dry-run regeneration check passes via `LearningSessionStore::dry_run_regeneration_check`.



