# Intent Router Latency Benchmarks

Benchmarks were recorded on a Ryzen 7 / 32 GB dev workstation using the chat-session harness (`tests/integration/intent_router_execute.rs`).  
Each run invoked `ChatSession::handle_message` with multi-intent payloads and captured wall-clock time via `Instant::now()` instrumentation embedded in the harness binary.

| Scenario | Base Size (events) | Intents per turn | Median (ms) | P95 (ms) | Notes |
|----------|-------------------|------------------|-------------|----------|-------|
| Small Base | 5 orchestration events | 2 (summary + profile) | 164 | 233 | Primarily parser/dispatcher overhead; no IO contention. |
| Medium Base | 50 events (light backlog) | 3 (summary + profile + suggestion) | 412 | 685 | Accounts for reading `knowledge.json` + consent manifests for suggestions. |
| Large Base | 500 events (heavy backlog) | 3 | 1 220 | 1 760 | Includes 0.4 s spent scanning `library_entries.json` and `ingestion` backlog. Still <2 s target. |

## Methodology
1. Seed Bases with deterministic orchestration history using the integration fixture.
2. Preload backlog data (`needs_pdf = true`) and knowledge entries to trigger suggestions.
3. Execute the harness binary 25 times per scenario, discard first 5 runs (warm-up), and compute median/P95 across remaining samples.
4. Verify `intent_detected/failed/executed/suggestion.snapshot` events were appended to `/AI/<Base>/intents/log.jsonl` for every run to confirm logging overhead stayed negligible.

## Takeaways
- Router stays within the 2 s SLA even when scanning 500-event Bases locally.
- Suggestion generation adds ~200–250 ms because it touches consent manifests and knowledge profiles; caching those paths brings the median back under 1 s.
- Confirmation/fallback paths remain dominated by chat IO, so no additional perf work is required until Bases exceed ~1 000 orchestration events.
