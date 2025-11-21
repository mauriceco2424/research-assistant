# Research: Learning Mode Sessions (008-learning-mode)

**Purpose**: Consolidate clarifications and rationale to unblock planning and design.
**Date**: 2025-11-21
**Spec**: specs/008-learning-mode/spec.md

## Decisions

- **Session question bound**: Default to five questions per session, with user able to continue or stop in chat.
  - **Rationale**: Keeps sessions lightweight, minimizes configuration friction, and preserves a bounded default for testing.
  - **Alternatives considered**: User-chosen question count (adds friction), time-boxed sessions (harder to test and monitor), unlimited until stop (risk of runaway loops).

- **Local-only by default**: All generation/evaluation must run locally unless explicit approval for external models is granted per batch (per P1/P2).
  - **Rationale**: Aligns with local-first privacy; prevents hidden network calls.
  - **Alternatives considered**: Transparent always-on remote calls (rejected: violates P1/P2), hybrid silent fallback (rejected: opaque behavior).

- **Orchestration logging + undo**: Log question generation, evaluation, KnowledgeProfile updates, and undos as events; allow undo of latest update.
  - **Rationale**: Required by P6; supports recovery from incorrect updates.
  - **Alternatives considered**: Silent updates without logs (rejected: violates P6), bulk undo only at session end (rejected: weaker control).

- **Regenerability**: Persist session artifacts (questions, evaluations, summaries) in AI-layer so sessions can be replayed/regenerated with Base contents.
  - **Rationale**: Required by P3/P4; ensures predictable reruns.
  - **Alternatives considered**: Ephemeral-only session state (rejected: breaks regenerability).

## Outstanding Questions

- None. All critical clarifications resolved in spec.
