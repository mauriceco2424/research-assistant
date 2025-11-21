
---

## Phase N+1: Success Criteria Validation

**Purpose**: Validate measurable outcomes for latency, manifests, provenance, and NEEDS_PDF handling.

- [ ] T025 [P] Add latency check for SC-001 (topic/gap/session request -> candidates under 30s) in tests/integration/discovery_latency.rs
- [ ] T026 [P] Add manifest/network audit for SC-005 (prompt manifests + endpoints logged; no hidden calls) in tests/integration/discovery_audit.rs
- [ ] T027 Validate provenance persistence for SC-003 (approved items persisted with consent/provenance) in tests/integration/discovery_provenance.rs
- [ ] T028 Validate NEEDS_PDF handling for SC-004 (failed fetch -> metadata-only + reason) in tests/integration/discovery_needs_pdf.rs
