# Implementation Plan: Chat Assistant & Intent Routing

**Branch**: `006-chat-intent-routing` | **Date**: 2025-11-20 | **Spec**: specs/006-chat-intent-routing/spec.md  
**Input**: Feature specification from `/specs/006-chat-intent-routing/spec.md`

## Summary

Deliver a conversational assistant that translates natural-language chat into orchestrated ResearchBase commands. The router must parse multi-intent utterances, halt downstream intents when earlier ones fail, confirm destructive or remote operations (honoring the global remote-disable toggle), manage a registry of capability descriptors, surface contextual suggestions, and log every intent event (detected/confirmed/executed/failed) so chat-driven workflows remain auditable and regenerable per P1�P10.

## Technical Context

**Language/Version**: Rust 1.75+ backend (orchestrator), TypeScript 5.x front-end (Tauri shell)  
**Primary Dependencies**: Existing `src/chat` + `src/orchestration` modules, `serde/serde_json`, `chrono`, `uuid`, integration harness under `tests/integration`, developer docs under `docs/`  
**Storage**: AI-layer JSONL intent logs under `/AI/<Base>/intents/`, existing orchestration log extended with intent events  
**Testing**: `cargo test` (unit + integration) with chat-session harness to simulate user flows  
**Target Platform**: Desktop (Windows/macOS/Linux) via Tauri shell  
**Project Type**: Single desktop project (Rust orchestrator + TS UI)  
**Performance Goals**: Intent parsing/confirmation responses <2s for Bases with ≤500 orchestration events; ≥95% success for intents with confidence ≥0.80  
**Constraints**: Local-first (no hidden network), explicit consent for remote inference, offline-capable when remote disabled  
**Scale/Scope**: Supports all current chat commands + future registrations; multi-intent routing per message; dozens of capability descriptors

## Constitution Check

| Principle | Compliance Plan |
|-----------|-----------------|
| **P1 Local-First** | Intent payloads, confirmation tickets, and suggestion context snapshots stored locally under each Base; router never calls remote services beyond existing consented commands. |
| **P2 Consent** | Prompt manifests summarized in chat for remote operations; approvals logged with `intent_confirmed` events and tied to orchestration undo tokens. |
| **P3 Dual-Layer** | Intent logs and confirmation outcomes written to AI-layer JSONL files so chat workflows can be replayed without in-memory state. |
| **P4 Regenerability** | Detected/confirmed/executed events capture deterministic payloads, enabling regeneration of chat sessions alongside existing orchestration history. |
| **P5 Chat-First** | Entire experience remains inside chat; confirmations/clarifications rendered inline, no new dashboards or panes. |
| **P6 Transparency** | Every routed intent surfaces event IDs, undo instructions, and failure reasons; destructive actions require explicit confirm phrases. |
| **P7 Integrity / P8 Learning** | Context suggestions cite AI-layer evidence (e.g., KnowledgeProfile flags) before recommending learning actions, preventing unsupported claims. |
| **P9 Versioning** | Intent schema versioned; capability descriptors include version metadata so downstream specs can evolve without breaking existing ones. |
| **P10 Extensibility** | Capability registry allows new features (Writing Assistant, Learning Mode) to add intents declaratively rather than editing router core. |

**Gate Result**: PASS (no violations).

## Project Structure

### Documentation (this feature)

```text
specs/006-chat-intent-routing/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md          # created later by /speckit.tasks
```

### Source Code (repository root)

```text
src/
├── chat/             # chat session + command handling
├── orchestration/    # orchestration events, profiles, learning hooks
├── bases/
├── acquisition/
└── reports/

tests/
├── integration/      # chat-session harness, new intent router coverage
└── unit/
```

**Structure Decision**: Single desktop workspace; router enhancements live in `src/chat` (intent parsing, confirmations, fallback UX) and `src/orchestration` (intent registry, logging). Integration tests extend existing chat harness in `tests/integration`.

## Complexity Tracking

*(No constitutional violations to justify; table intentionally empty.)*
