# Quickstart - AI Profiles & Long-Term Memory

## Prerequisites
- ResearchBase desktop workspace cloned locally with Tauri prerequisites installed.
- Existing Base initialized by Specs 01-04 (onboarding, ingestion, etc.).
- `cargo` + `pnpm` (or npm) available for backend/frontend builds.

## 1. Seed a Test Base
1. Launch the app or run CLI to create a Base (e.g., `bases create demo-base`).
2. Ensure onboarding captured initial profile hints (name, tone, projects). If not, run `profile interview user` to seed defaults.

## 2. Run Profile Show/Edit Loop
```bash
researchbase chat "profile show writing"
researchbase chat "profile update work focus=\"Submit CHI draft\""
```
- Confirm chat response includes structured JSON snippet, timestamps, audit pointer.
- Verify `/AI/demo-base/profiles/work.json` updated and `history` appended.

## 3. Test Guided Interview with Remote Consent
1. Copy a local PDF snippet into `/User/demo-base/papers`.
2. Run `profile interview writing` and accept the consent manifest when prompted.
3. Inspect `/AI/demo-base/consent/manifests/<id>.json` and the resulting `writing.json` remote metadata block.

## 4. Audit & Undo
```bash
researchbase chat "profile audit writing"
researchbase chat "profile undo writing <event_id>"
```
- Validate audit list references orchestration events, consent manifests, and undo instructions.

## 5. Export, Delete, Regenerate
```bash
researchbase chat "profile export knowledge"
researchbase chat "profile delete knowledge"
researchbase chat "profile regenerate --from-history knowledge"
```
- Confirm ZIP archive created under `/User/demo-base/profiles/exports`.
- After delete, JSON/HTML removed but audit entry created.
- Regeneration should replay events and report matching hash; if history missing, ensure graceful error.

## 6. Integration Hooks
- Call `profile.get_work_context()` from planning workflows (tests under `tests/integration/profile_work_context.rs`).
- Call `profile.get_knowledge_summary()` before entering Learning Mode to confirm summary formatting.

## 7. Testing Strategy
- **Unit tests**: Validate schema serialization/deserialization, hash calculators, consent manifest parser.
- **Integration tests**: End-to-end chat simulations writing to temp Base directories (use `tempfile` crate).
- **Manual validation**: Inspect HTML summaries, confirm locks prevent concurrent export + update collisions.

## Troubleshooting
- Missing profile file -> run interview command; plan ensures defaults created during migrations.
- Remote inference disabled -> provide user message referencing P2 toggle.
- Hash mismatch on regeneration -> direct user to `profile audit` plus restore from backup ZIP.
