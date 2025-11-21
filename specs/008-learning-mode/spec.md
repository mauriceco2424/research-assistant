# Feature Specification: Learning Mode Sessions

**Feature Branch**: `008-learning-mode`  
**Created**: 2025-11-21  
**Status**: Draft  
**Input**: User description: "Introduce Learning Mode (Spec 08) for quizzes and oral exams driven by the KnowledgeProfile, aligned with master_spec.md 9 and constitutional P1-P10. The user wants to select scope (entire Base, specific categories, specific papers, or concepts) and choose a mode (Quiz vs Oral Exam). The system should generate questions based on KnowledgeProfile gaps, evaluate answers with corrective feedback, update KnowledgeProfile progress, and surface recommendations for further study. Must remain chat-first (no new complex UI), local-first, and transparent: log orchestration events and allow undo of KnowledgeProfile updates. Success criteria: (1) user can start a learning session via chat with a chosen scope/mode; (2) questions reflect KnowledgeProfile coverage/difficulty; (3) evaluations include corrections and suggested follow-ups; (4) KnowledgeProfile is updated per interaction with explicit event logging; (5) workflow is regenerable from AI-layer data, no hidden network calls."

## Clarifications

### Session 2025-11-21

- Q: How should question count be bounded per session by default? → A: Default 5 questions; user can continue or stop in chat.

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Start a scoped learning session via chat (Priority: P1)

The researcher starts a learning session from chat, selecting scope (entire Base, chosen categories, selected papers, or specific concepts) and mode (Quiz or Oral Exam). The system confirms scope/mode, states that processing stays local with no hidden network calls, and initializes the session.

**Why this priority**: Without reliable session kickoff and clear scope/mode selection, no downstream learning experience is possible.

**Independent Test**: Start a session from an existing Base via chat, select scope and mode, and verify the session is created with a visible confirmation and stored context even if no further steps run.

**Acceptance Scenarios**:

1. **Given** a Base with KnowledgeProfile data, **When** the user requests a Quiz for selected categories, **Then** the system confirms scope/mode and opens a session context visible in chat.
2. **Given** a Base with KnowledgeProfile data, **When** the user requests an Oral Exam for specific papers, **Then** the system acknowledges the papers and starts the session without opening new UI views.

---

### User Story 2 - Receive and answer targeted questions with feedback (Priority: P2)

During a session, the system generates questions that reflect KnowledgeProfile gaps and difficulty, presents them in chat, captures the user's answers, and returns corrective feedback plus recommended follow-ups.

**Why this priority**: Question/answer with corrective guidance is the core learning loop and must align to KnowledgeProfile coverage.

**Independent Test**: Run a session with a small scoped Base, answer at least one question, and confirm the feedback cites the relevant concepts and suggests next steps without needing other features.

**Acceptance Scenarios**:

1. **Given** a session in Quiz mode, **When** the system asks a question from under-covered concepts, **Then** the user's answer is evaluated and feedback cites the missed concepts with suggested follow-ups.
2. **Given** a session in Oral Exam mode, **When** the user responds verbally via chat, **Then** the system provides structured evaluation (strengths, gaps, recommendations) in the same chat flow.

---

### User Story 3 - Review and adjust KnowledgeProfile updates (Priority: P3)

After or during a session, the researcher reviews how answers changed the KnowledgeProfile (coverage, competence, difficulty metadata) and can undo the latest applied updates with orchestration logging preserved.

**Why this priority**: Transparency and undoability are constitutional requirements (P6) and protect users from incorrect updates.

**Independent Test**: Complete a session, inspect logged updates, undo the latest change, and confirm the KnowledgeProfile and logs reflect the reversal without needing other stories.

**Acceptance Scenarios**:

1. **Given** a completed session, **When** the user requests a summary, **Then** the system shows which concepts were updated and the associated evaluation rationale.
2. **Given** recent updates from the session, **When** the user issues an undo command for the latest change, **Then** the KnowledgeProfile reverses that change and records the undo event.

---

### Edge Cases

- Scope contains no covered materials (empty KnowledgeProfile segment): session creation should inform the user and offer to switch scope or ingest more content instead of proceeding with empty questions.
- User cancels mid-session: stop question generation, log a cancellation event, and preserve progress so the session can be resumed or discarded explicitly.
- No network permitted or transient offline state: all prompts and evaluations proceed locally; if an external model is requested, the system must block and explain why (per P1/P2) rather than silently failing.
- Undo requested after multiple updates: only the latest update is reversed per command, and the log reflects both the original change and the undo.
- Long-running evaluation or generation stalls: inform user in chat, allow abort, and avoid partial KnowledgeProfile updates.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: The user MUST be able to start a learning session from chat by selecting scope (entire Base, chosen categories, selected papers, or concepts) and choosing mode (Quiz or Oral Exam) without leaving chat.
- **FR-002**: The system MUST confirm scope, mode, and local-only processing before beginning, and store this session context for later reference or regeneration.
- **FR-003**: The system MUST generate questions from KnowledgeProfile coverage/difficulty gaps for the chosen scope and present them in chat, avoiding any hidden network calls; if an external model is needed, the system MUST request explicit approval per batch.
- **FR-004**: The system MUST capture each user answer and evaluate it with corrective feedback that identifies missed or partially met concepts and suggests targeted follow-ups (papers, concepts, or exercises) within the chosen scope.
- **FR-005**: The system MUST adapt question difficulty within a session based on prior answers and KnowledgeProfile signals so that under-covered or weak areas are prioritized.
- **FR-005a**: Each session MUST default to five questions and allow the user to continue with additional questions or stop early via chat.
- **FR-006**: The system MUST update the KnowledgeProfile after each evaluated answer (e.g., coverage, competence, difficulty metadata) and tie each update to the triggering session/question.
- **FR-007**: The system MUST log orchestration events for question generation, evaluation, KnowledgeProfile updates, and undo actions, including timestamps and scope/mode metadata (P6).
- **FR-008**: The user MUST be able to undo the latest KnowledgeProfile update from the session via chat, with the reversal recorded as a new orchestration event.
- **FR-009**: The system MUST provide a session summary in chat that recaps asked questions, evaluation outcomes, KnowledgeProfile changes, and recommended next study actions, and it MUST be regenerable from AI-layer data plus Base contents (P3/P4).
- **FR-010**: The system MUST keep all artifacts (questions, responses, evaluations, logs) stored locally under the Base, ensuring local-first privacy (P1) and no hidden network access (P2).

### Key Entities *(include if feature involves data)*

- **LearningSession**: Scope (Base/categories/papers/concepts), mode (Quiz/Oral Exam), status, start/end timestamps, event log references, regeneration pointers.
- **QuestionItem**: Prompt text, targeted concepts/papers, difficulty metadata, rationale for selection, expected answer outline for evaluation.
- **ResponseEvaluation**: User answer, evaluation outcome (correct/partial/incorrect), corrective feedback, follow-up recommendations, linked KnowledgeProfile updates.
- **KnowledgeProfile Entry**: Concept/paper/category coverage, competence level, difficulty signals, history of updates and undo markers tied to session/question IDs.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: Users can initiate a scoped learning session via chat (scope + mode confirmed) in <=2 chat turns for at least 95% of attempts.
- **SC-002**: 90% of generated questions in a session target KnowledgeProfile gaps or under-covered difficulty tiers within the chosen scope.
- **SC-003**: 100% of evaluated answers return corrective feedback with at least one recommended follow-up action in the same chat turn.
- **SC-004**: KnowledgeProfile updates from sessions are logged with scope/question context and can be undone for 100% of updates issued within the session.
- **SC-005**: All session artifacts (questions, evaluations, summaries, logs) are regenerable from AI-layer data plus Base contents with zero hidden network calls, confirmed in a dry-run regeneration test.

### Assumptions

- Sessions run entirely within the existing chat surface; no new persistent UI views will be introduced (P5).
- KnowledgeProfile structure and storage already exist; this feature reads/writes within that schema without destructive migrations (P9/P10).
- Learning questions rely only on locally available Base content and authorized AI-layer data; any external model use requires explicit user approval per batch (P1/P2).

