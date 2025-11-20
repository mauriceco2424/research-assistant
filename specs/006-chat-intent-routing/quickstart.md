# Quickstart – Chat Assistant & Intent Routing

## Prerequisites
- Existing ResearchBase workspace compiled with latest orchestrator changes.
- At least one Base seeded with papers/profiles so commands produce meaningful output.
- Integration harness available (`cargo test` runs clean).

## 1. Enable the Assistant Router (Development Mode)
1. Checkout branch `006-chat-intent-routing`.
2. Build and run the desktop app (`cargo tauri dev` or equivalent).
3. Open/Select a Base in chat to initialize the session.

## 2. Test Multi-Intent Execution
1. Type: `Summarize the last 3 papers and show my writing profile`.
2. Expect two chat responses: confirmation of summary scope + notification that profile view will follow.
3. Verify orchestration log entries `intent_detected` → `intent_executed` for both actions.

## 3. Validate Confirmation Safeguards
1. Type: `Delete the writing profile`.
2. Assistant should list eligible profiles, display confirm phrase, and require `DELETE writing`.
3. Approve the prompt; confirm chat output includes event id, undo instructions, and confirmation ticket stored under `/AI/<Base>/intents`.

## 4. Exercise Clarification & Fallback
1. Issue ambiguous request: `Please handle that task`.
2. Assistant should respond with clarifying options or fallback instructions to run manual commands.
3. Ensure `intent_failed` events capture low-confidence cases.

## 5. Remote Consent Flow
1. Disable remote inference; attempt: `Use AI to infer my writing tone`.
2. Assistant must state remote calls are blocked.
3. Re-enable remote inference, re-run command, and approve the prompt manifest; consent id recorded in confirmation ticket.

## 6. Contextual Suggestions
1. Seed pending consent manifests or mark KnowledgeProfile entries as STALE.
2. Ask `What should I do next?`
3. Assistant should cite the pending evidence (e.g., consent ids, stale concepts) when suggesting next actions.

## 7. Troubleshooting
- **Router unresponsive**: Check intent log under `/AI/<Base>/intents/log.jsonl` for parsing errors.
- **Confirmations not appearing**: Ensure Base selection is active; tickets are Base-specific.
- **Undo unavailable**: If action is older than retention policy, assistant should point to audit log instructions documented in chat.
