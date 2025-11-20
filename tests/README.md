# Test Evidence – Spec 005 AI Profiles

## Automated Suites
| Command | When (UTC) | Result |
|---------|------------|--------|
| `cargo test profile_governance` | 2025-11-19T00:00Z | PASS |
| `cargo test` | 2025-11-19T00:00Z | PASS |
| `cargo test` | 2025-11-20T17:11:43Z | PASS |

## Manual / Chat Validation
- Followed the updated Quickstart steps via the integration harness (`ProfileBaseFixture`) to emulate `profile show/update`, `profile audit`, `profile export/delete/regenerate`, and `profile scope` flows.
- Confirmed audit output lists diff summaries + hashes, export lock surfaces `EXPORT_IN_PROGRESS`, delete removes JSON/HTML artifacts, regenerate restores state, and scope commands log `ScopeChange` orchestration events.
- Exercised Phase 4–5 chat flows (`handle_message("What should I do next?")`, destructive deletes, ambiguous requests) to verify clarification prompts, suggestion snapshots, manual command hints, and confirmation ticket persistence under `/AI/<Base>/intents`.
