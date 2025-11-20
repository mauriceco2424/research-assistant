# Profile Show Latency (Spec 005)

Measured with `cargo run --bin profile_show_bench -- <entries>` on 2025-11-19 (Windows, debug build).

| Entries | Duration | SC-001 Target (≤5s p95) | Result |
|---------|----------|-------------------------|--------|
| 5       | 0.00036 s | ✓ | PASS |
| 50      | 0.00214 s | ✓ | PASS |
| 500     | 0.00367 s | ✓ | PASS |

Notes:
- Bench harness seeds deterministic KnowledgeProfile data before calling `ProfileService::show` twice (warm-up + measured) to exercise HTML rendering and scope enforcement.
- All scenarios stay well below the 5-second SLA; even 500-entry runs complete under 4 ms on this machine, leaving ample headroom for heavier HTML formatting or additional history projections.
