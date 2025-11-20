# Spec Argument – Chat Assistant & Intent Routing (Spec 06)

Design the **Chat Assistant + Intent Routing** capability described in `master_spec.md` §3 (Chat Panel) and §14 (Intent Routing & Orchestration), aligned with roadmap item **Spec 06**. Produce a spec that formalizes how natural-language inputs are parsed, confirmed, and dispatched to the existing command surfaces (profiles, reports, ingestion, learning prep, etc.) while preserving constitutional guarantees (P1–P10) and the dual-layer logging requirements established in prior specs.

## User Intent & Scenarios
- **Natural Conversation to Commands**: A researcher types “Summarize the last 3 papers I imported and then show my writing profile.” The assistant must detect multiple intents, confirm if the workflow is destructive or long-running, and queue/execute the corresponding orchestration calls (report generation, `profile show writing`) without the user memorizing CLI syntax.
- **Clarification & Disambiguation**: When a user says “delete the profile,” the assistant should clarify which profile/base, highlight the confirm phrase, and surface constitutional constraints (P6 transparency, P1 local-first) before calling `profile delete`.
- **Mixed Context Awareness**: Before suggesting actions such as new paper discovery or learning mode, the assistant inspects AI-layer signals (profile scopes, last ingestion timestamp, pending consent items) and offers options (“I can fetch papers, refresh categories, or prep a learning session”) with explicit confirmations.
- **Orchestration Feedback Loop**: After routing a command, the assistant reports progress updates, surface orchestration event IDs, and instructions for undo/redo so the chat log becomes the authoritative audit trail.

## Constraints & Constitutional Alignment
- **P1 Local-First / P2 Consent**: The router must never trigger remote inference or acquisition without summarizing the prompt manifest and capturing explicit approval. Cached intents must live locally (no cloud queues).
- **P3/P4 Dual-Layer**: Every routed action logs an orchestration event (intent id, parsed command, confirmation tokens) so regenerations can recreate the chat decision tree. Chat transcripts remain the user-layer artifact; intent manifests live in AI-layer JSON.
- **P5 Chat-First**: No new GUI; all routing, confirmations, and progress updates appear inline in chat. Buttons/links are limited to simple quick-reply affordances, not new panels.
- **P6 Transparency / Undoability**: Non-trivial operations require explicit confirmation (especially destructive tasks). The assistant must surface event IDs + undo instructions in its replies, and bulk actions need “Are you sure?” loops.
- **P7 Integrity / P8 Learning**: When routing to knowledge or learning flows, the assistant cites the evidence it will use (e.g., KnowledgeProfile entries marked STALE) and labels speculative suggestions as such.
- **P9/P10 Versioning & Extensibility**: Intent schemas must be versioned and easily extensible so future specs (Writing Assistant, Learning Mode) can register new intents without breaking existing ones.

## Success Criteria
1. **Intent Schema & Router**: Define a deterministic JSON schema for parsed intents (action, target, parameters, safety classification) and the routing engine that maps chat utterances to concrete command handlers or follow-up questions.
2. **Confirmation & Safety Flow**: Specify how the assistant confirms high-impact actions (delete, export, remote inference) including default prompts, timeout behavior, and how confirmations are persisted/logged.
3. **Ambiguity & Error Recovery**: Describe fallback interactions when the assistant cannot confidently map an intent (e.g., ask clarifying questions, offer suggestions, defer to manual command syntax).
4. **Orchestration Logging**: Enumerate the orchestration events emitted by the router (intent_detected, intent_confirmed, intent_failed) with required metadata (chat turn id, Base id, consent manifest ids if any).
5. **Extensibility Contracts**: Provide registration contracts so features like Reports, Profiles, Ingestion, Learning Mode can expose their capabilities/validation rules to the router without tight coupling.

## Scope Guardrails
- Do **not** implement the downstream features themselves (reports, writing assistant, learning mode); limit the spec to routing, confirmation, and orchestration glue plus minimal capability discovery APIs.
- Maintain compatibility with existing chat commands—the router should generate or invoke them, not replace them.
- No background automation; the assistant must stay user-driven (per P5/P6) and cannot enqueue hidden jobs without chat-visible confirmation.
- Explicitly call out risks (e.g., intent misclassification leading to destructive actions) and mandate mitigations such as confidence thresholds, user confirmation loops, and undo guidance.
