# Spec Argument – AI Profiles & Long-Term Memory (Spec 05)

Design the **AI Profiles** capability described in master_spec.md §7 and roadmap item Spec 05. Produce a spec that defines how the system captures, stores, surfaces, and updates the four long-term profiles (UserProfile, WorkProfile, WritingProfile, KnowledgeProfile) entirely within the AI layer, aligning with constitutional principles (P1–P10) and building on the completed Specs 01–04.

## User Intent & Scenarios
- **Profile Interview & Review**: After onboarding, the researcher can run chat flows such as profile show user or profile update writing to inspect/edit structured JSON describing their background, tone, and goals without digging into files.
- **Context-Aware Assistance**: When asking for writing or planning help, the user expects the AI to reuse stored preferences (tone, project TODOs, concept mastery) rather than re-asking. They also need profile scope <Base> and profile export/delete to control where the AI may apply the data.
- **Knowledge Tracking**: Before entering Learning Mode, the user wants to view the KnowledgeProfile (concept → mastery → evidence), mark strengths/weaknesses, and link entries to papers/notes, all from chat.
- **Audit & Undo**: Commands like profile audit work should list when/why entries changed, referencing orchestration event IDs and offering undo guidance so users trust the long-term memory.

## Constraints & Constitutional Alignment
- **P1 Local-First**: Profiles live in /AI/<Base>/profiles/*.json with optional HTML summaries in /User/<Base>/profiles/; no silent network sync.
- **P2 Consent**: Any remote summarization (e.g., extracting writing tone from PDFs) must emit a prompt manifest, request consent, and log approvals before touching external endpoints.
- **P3/P4 Dual-Layer**: Profiles are deterministic artifacts (JSON/Markdown) regenerated from chat inputs + orchestration logs via profile regenerate --from-history.
- **P5 Chat-First**: All interactions (interviews, edits, exports) happen through chat commands or generated HTML summaries; no new UI panes.
- **P6 Transparency**: Every profile mutation logs an orchestration event (who/what/when) and exposes undo instructions.
- **P7 Integrity / P8 Learning**: KnowledgeProfile entries must cite evidence (paper IDs, notes) and clearly label uncertainties so future learning sessions stay academically honest.

## Success Criteria
1. Chat commands profile show <user|work|writing|knowledge> render structured summaries with timestamps, evidence references, and edit history pointers.
2. Guided interviews (profile run writing-style, profile interview knowledge) collect data, confirm before overwriting, and store results locally with consent manifests when remote inference is needed.
3. KnowledgeProfile maintains concept mastery records (concept, mastery level, evidence, weaknesses) and exposes APIs/events that Learning Mode can consume later.
4. Running profile export/profile delete obeys privacy expectations (local ZIP export, per-profile wipe), and profile regenerate --from-history rebuilds identical JSON from orchestration logs, proving regenerability.

## Scope Guardrails
- Do **not** implement the full Chat Assistant intent router or Learning Mode; instead, define the contracts they will call (e.g., profile.get_work_context()).
- Reuse existing onboarding data where possible; specify migrations or defaults for existing Bases so they start with minimal profiles.
- Keep the spec implementation-agnostic beyond storage layout, consent hooks, commands, and orchestration requirements so /speckit.plan can assign modules/tests.
- Call out risks (e.g., remote style extraction, stale mastery data) and mandate mitigation steps such as consent manifests, audit logs, and user confirmation flows.
