# AGENTS – ResearchBase Workspace Guide

This file describes how agents should work in this repository: core context,
non‑negotiable constraints from the constitution, and how to use the SpecKit
workflow (especially `/speckit.specify`).

Scope: This `AGENTS.md` applies to the entire repository.

---

## 1. Project Overview

- **Project name**: ResearchBase  
- **Goal**: Desktop‑first, AI‑orchestrated research environment to help
  researchers:
  - organize, understand, and expand their literature,
  - maintain long‑term structured knowledge,
  - write and revise academic work (LaTeX‑friendly),
  - learn and master material via interactive sessions,
  - operate from a minimal, chat‑first interface.
- **Key reference docs**:
  - `master_spec.md` – master product and behavior specification.
  - `.specify/memory/constitution.md` – ResearchBase Constitution v1.0.0.
  - `const_arg.md` – binary, measurable principles used to derive the
    constitution.

When in doubt, the *constitution* and *master spec* are the sources of truth.

- **Current implementation stack**: Tauri desktop shell (Rust backend + TypeScript/HTML front-end). Treat this as the default for new specs/plans unless governance approves a change.

---

## 2. Core Constitutional Constraints (Agent Summary)

Agents MUST respect the following (see `.specify/memory/constitution.md` for
details):

- **P1 – Local‑First Privacy**
  - All research artifacts stay on the local filesystem under a user‑selected
    base directory.
  - No hidden network calls; external endpoints must be explicit.

- **P2 – Explicit AI Data Sharing, Consent, and Acquisition Approval**
  - Any off‑device AI call must be described by a prompt manifest and approved
    per Base.
  - Paper acquisition (DOI/URL → metadata/PDF) requires explicit per‑batch user
    approval; this approval must be logged.

- **P3/P4 – Dual‑Layer Architecture & Regenerability**
  - User Layer: PDFs, LaTeX, HTML reports.
  - AI Layer: structured JSON/Markdown memory.
  - All reports and complex artifacts must be regenerable from AI Layer + Base
    contents.

- **P5 – Minimal, Chat‑First Interface**
  - Chat is the primary interaction surface.
  - No new complex permanent UI views; complexity belongs in chat and HTML
    reports.

- **P6 – Orchestration Transparency**
  - Non‑trivial AI operations must be logged as orchestration events.
  - Bulk/destructive operations require explicit confirmation and should be
    undoable (at least last N operations).

- **P7–P10 – Academic Integrity, Learning, Versioning, Extensibility**
  - No silent citation fabrication; unverified references must be clearly
    marked.
  - Learning modes and evidence‑backed assistance are required.
  - Constitution/specs must be versioned and kept aligned.
  - New features must not break existing Bases or force destructive migrations.

Agents MUST NOT introduce features, behaviors, or specs that violate these
principles unless an explicit, documented exemption is granted (see P9).

---

## 3. SpecKit Workflow in This Repo

SpecKit lives under `.specify/` and `.codex/prompts/`. The typical flow for a
new feature is:

1. **Constitution awareness**
   - Read `.specify/memory/constitution.md` and `master_spec.md` before major
     changes.
   - Ensure you understand P1–P10 and the acquisition workflow.

2. **Feature Specification – `/speckit.specify`**
   - Takes a human description of *one* feature or tightly scoped change.
   - Produces a feature spec under `specs/[###-feature-name]/spec.md` using
     `.specify/templates/spec-template.md`.

3. **Implementation Plan – `/speckit.plan`**
   - Consumes the spec and fills `plan.md` from
     `.specify/templates/plan-template.md`.
   - Must perform a **Constitution Check** against P1–P10.

4. **Task Breakdown – `/speckit.tasks`**
   - Uses the spec + plan to generate `tasks.md` from
     `.specify/templates/tasks-template.md`.
   - Tasks are grouped by user story and must include any work necessary to
     satisfy constitutional requirements (logging, local‑first storage,
     regenerability, chat‑first interaction, acquisition approval, etc.).

5. **Other commands (`.codex/prompts/speckit.*.md`)**
   - `/speckit.analyze`, `/speckit.clarify`, `/speckit.checklist`,
     `/speckit.implement`, etc., are helpers, but they must not contradict the
     constitution or master spec.

Agents using these commands should always keep the constitution and master spec
in view and call out any conflicts explicitly.

---

## 4. Guidance for `/speckit.specify` Arguments

The argument to `/speckit.specify` is critical: it seeds the feature spec and
downstream plan/tasks. Treat it as the *single‑feature problem statement* plus
constraints, not as a dump of everything.

### 4.1 What the argument SHOULD contain

- **One focused feature or change**
  - Example: “Introduce the Paper Acquisition Workflow for Path B onboarding
    and on‑demand discovery, aligned with the constitution’s consent rules.”

- **User‑level intent and scenarios**
  - What the user is trying to achieve (e.g., “I want to accept AI‑suggested
    papers and have the app fetch PDFs like Zotero, but always with explicit
    approval”).

- **Relevant constraints from the master spec / constitution**
  - E.g., “Must remain chat‑first,” “must not auto‑download PDFs without
    per‑batch approval,” “must keep User/AI layers in sync.”

- **Success criteria**
  - What it means for the feature to succeed: explicit, measurable outcomes
    (e.g., “user can add 10 papers via DOI in one batch with a single approval
    step and see which ones still need manual PDFs”).

- **Context references**
  - References to sections in `master_spec.md` that the feature refines or
    extends (e.g., Path A, Path B, Paper Discovery, Non‑Functional Principles).

### 4.2 What the argument SHOULD NOT contain

- **Multiple unrelated features**
  - Avoid mixing, e.g., “new acquisition workflow” + “new visualization
    system” + “rewrite of learning mode” in one `/speckit.specify` call.

- **Low‑level implementation detail**
  - Do not prescribe file names, class names, or internal module layout unless
    absolutely necessary. Leave structure to the plan/tasks stages.

- **Requests that contradict P1–P10**
  - No background auto‑harvesting of PDFs without user approval.
  - No new dashboard‑style complex UI views bypassing chat + HTML reports.
  - No “cloud‑first” assumptions that break local‑first privacy.

- **Huge code dumps**
  - Only include code snippets if they are essential to clarify an interface or
    contract. The goal of `/speckit.specify` is requirements and behavior, not
    pasting entire modules.

- **Ambiguous goals**
  - Avoid vague phrases like “make it better” or “improve UX” without stating
    what “better” means (e.g., faster onboarding, fewer clicks, reduced
    confusion about acquisition).

### 4.3 Alignment requirements

When crafting `/speckit.specify` arguments, agents MUST:

- Explicitly state any relevant constitutional principles (P1–P10) and how the
  feature must comply.
- Respect the **Paper Acquisition Workflow** pattern:
  - AI proposes → user explicitly approves → app attempts acquisition →
    failures become metadata‑only `NEEDS_PDF` entries with clear feedback.
- Ensure the feature remains **chat‑first**, **local‑first**, and
  **regenerable**.
- Call out if the user is *intentionally* asking for an exemption and mark it
  as such so P9 (spec alignment / exemptions) can be applied later.

---

## 5. General Agent Conduct

- Prefer **small, targeted changes** that keep the system aligned with the
  constitution and master spec.
- When editing specs or templates, always:
  - check `.specify/memory/constitution.md`,
  - document how your change affects P1-P10.
- Do not add UI complexity, network behavior, or storage mechanisms that
  bypass existing architectural rules.
- When unsure, propose the trade-off explicitly in text rather than silently
  "bending" the constraints.
- **Never rewrite history without explicit approval.** If the local branch is
  ahead of the remote (e.g., push failed), keep the local commits. Do **not**
  run `git reset`, `git revert`, or remove user work unless the user explicitly
  requests it. Instead, inform the user and wait for guidance.

