# ResearchBase – AI Profiles & Long-Term Memory

## 2025-11-19 – Profile Governance & Polish
- Added chat-first governance commands (`profile audit/export/delete/regenerate/scope`) with deterministic hashes, consent pointers, and scope enforcement per Spec 005.
- Documented Quickstart guidance for exports, delete/regenerate recovery, and scope updates, emphasizing export locks to keep archives deterministic and local-first.
- Captured integration coverage via `tests/integration/profile_governance.rs`, validating consent logging, orchestration replay, and scope changes before handing off to Phase 7 benchmarks.
