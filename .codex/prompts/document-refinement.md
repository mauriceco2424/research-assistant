---
description: Document a completed refinement with proper tracking in refinements.md and git history
---

## Overview

The `/document-refinement` command documents post-implementation fixes that were completed after `/speckit.implement` finished. It infers the spec, issue, and changes from conversation context and git history, then creates proper documentation and commits.

**Typical workflow:**
1. `/speckit.implement` completes all tasks
2. User tests and discovers issues
3. User works with Claude to fix issues (normal conversation, plan mode, whatever works)
4. User verifies fixes work
5. User runs `/document-refinement`
6. Command analyzes context, creates/updates refinements.md, and commits

**This command:**
- Requires NO arguments (infers everything from context)
- Creates/updates `specs/{spec-id}/refinements.md` (dedicated refinement tracking)
- Creates git commit with "Refine:" prefix and R-number
- Maintains traceability for future developers and AI agents

## Prerequisites

Before running this command:
- ✓ You have uncommitted changes (the refinement fixes)
- ✓ The fixes have been tested and confirmed working
- ✓ The conversation contains context about what issue was fixed

If git status is clean, the command will abort with: "No changes to document."

## Execution Flow

### Step 1: Validate Preconditions

Check git status:
```bash
git status
```

If working tree is clean:
```
❌ No changes to document.

Please make refinement fixes first, then run /document-refinement to document them.
```

If there are uncommitted changes, proceed.

### Step 2: Infer Context from Conversation

Analyze the conversation history to identify:
- **Which spec** is being refined (look for mentions like "spec 001", "minimum-playable", etc.)
- **What issue** was being addressed (user's problem description)
- **Root cause** (what you discovered during investigation)
- **Solution** (what was changed to fix it)

If spec cannot be determined from conversation, search git diff for clues:
```bash
git diff --name-only
```

Example: If files are in `specs/001-*/`, infer spec is "001".

If still unclear, ask user:
```
Which spec are these fixes for?
- 001 (minimum-playable)
- 002 (next-feature)
- [other]
```

### Step 3: Analyze Git Changes

Get list of modified files:
```bash
git diff --name-only
git diff --cached --name-only
```

Summarize what changed:
```bash
git diff
git diff --cached
```

Analyze the diff to understand:
- Which files were modified
- What specific changes were made
- How the changes address the issue

### Step 4: Generate Refinement Summary

Draft a refinement entry based on:
- Conversation context (issue description, root cause)
- Git diff analysis (what changed)
- Spec context (which spec is being refined)

Check if `specs/{spec-id}/refinements.md` exists:
```bash
ls specs/{spec-id}/refinements.md 2>/dev/null
```

If it exists, find the last refinement number:
```bash
grep -E "^## R[0-9]+" specs/{spec-id}/refinements.md | tail -1
```

Example: If last refinement is R002, next is R003.

If no refinements.md exists yet, start with R001.

### Step 5: Present Summary to User

Display the inferred refinement details:

```
📋 Refinement Summary (Spec {spec-id}, R{number})

**Issue:**
{issue_description_from_conversation}

**Root Cause:**
{root_cause_from_investigation}

**Solution:**
{solution_from_git_diff}

**Files Modified:**
- {file1}: {description}
- {file2}: {description}
...

Is this correct?
- yes: Proceed with documentation and commit
- edit: Let me correct the details
- cancel: Abort refinement documentation
```

Wait for user response.

### Step 6: Handle User Response

**If "yes"**: Proceed to Step 7

**If "edit"**: Ask user which field to edit:
```
Which details need editing?
- issue: Change issue description
- cause: Change root cause
- solution: Change solution description
- files: Adjust file descriptions
- all: I'll provide full details
```

Let user provide corrections, then return to Step 5 to confirm again.

**If "cancel"**: Abort and exit:
```
Refinement documentation cancelled. Your changes are still uncommitted.
Run /document-refinement again when ready.
```

### Step 7: Create/Update refinements.md

**If `specs/{spec-id}/refinements.md` doesn't exist**, create it with header:
```markdown
# Refinements: {Spec Title} (Spec {id})

This file tracks post-implementation fixes discovered after `/speckit.implement` completed.
Each refinement is documented with root cause analysis and solution details.

---

## R001: {Brief Title}
**Date:** {YYYY-MM-DD}
**Issue:** {user_description}
**Root Cause:** {investigation_findings}
**Solution:** {what_was_changed_and_why}
**Files Modified:**
- {file}: {specific_changes}
- {file}: {specific_changes}

**Commit:** {will_be_filled_after_commit}
```

**If file already exists**, append the new refinement entry (with separator):
```markdown

---

## R{number}: {Brief Title}
**Date:** {YYYY-MM-DD}
**Issue:** {user_description}
**Root Cause:** {investigation_findings}
**Solution:** {what_was_changed_and_why}
**Files Modified:**
- {file}: {specific_changes}
- {file}: {specific_changes}

**Commit:** {will_be_filled_after_commit}
```

### Step 8: Create Git Commit

Stage all changes including the documentation:
```bash
git add .
```

Create commit message file to avoid heredoc issues on Windows:
```bash
# Write commit message to temp file, then commit using -F flag
git commit -F .git/COMMIT_MSG_TEMP
```

Commit message format:
```
Refine: {Brief description} (Spec {spec-id}, R{number})

**Issue:**
{issue_description}

**Root Cause:**
{root_cause}

**Solution:**
{solution_description}

**Impact:**
{what_this_fixes_or_improves}

Files Modified:
- {file}: {changes}
- {file}: {changes}
- specs/{spec-id}/refinements.md: Documented refinement R{number}

Refinement of spec {spec-id}. This fix was tested and confirmed working by user.
See specs/{spec-id}/refinements.md for full refinement history.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Step 9: Update refinements.md with Commit SHA

After commit succeeds, get the commit SHA:
```bash
git rev-parse --short HEAD
```

Update the refinement entry in `specs/{spec-id}/refinements.md`:
- Replace `**Commit:** {will_be_filled_after_commit}`
- With `**Commit:** {actual_SHA}`

Stage and amend:
```bash
git add specs/{spec-id}/refinements.md
git commit --amend --no-edit
```

Note: The SHA will change after amend. This is expected and acceptable since the refinements.md now contains the pre-amend SHA which is close enough for traceability.

### Step 10: Ask About Push

```
✅ Refinement R{number} documented and committed successfully

**Commit:** {SHA}
**Spec:** {spec-id}
**Documented In:** specs/{spec-id}/refinements.md

Push to remote? (yes/no)
```

If "yes":
```bash
git push
```

If "no":
```
Changes committed locally. Run 'git push' manually when ready.
```

### Step 11: Report Completion

```
✅ Refinement Complete (Spec {spec-id}, R{number})

**Issue:** {brief_description}
**Files Modified:** {count} files
**Documented In:** specs/{spec-id}/refinements.md

**Commit:** {SHA}

Ready for next refinement or new spec.
```

## Error Handling

### No Changes to Commit
```
❌ No uncommitted changes found.

Please make refinement fixes first, then run /document-refinement.
```

### Cannot Determine Spec
```
❌ Cannot determine which spec these changes are for.

Please specify: Which spec are you refining?
- 001 (minimum-playable)
- 002 (another-feature)
```

### Spec Directory Not Found
```
❌ Spec directory not found: specs/{spec-id}-*

Available specs:
- 001-minimum-playable
- 002-another-feature

Which spec should I document this refinement for?
```

### Commit Fails
```
❌ Commit failed: {error_message}

Your changes and documentation are preserved.
Troubleshoot the git error and run /document-refinement again.
```

### Cannot Infer Context
```
❌ Cannot infer refinement details from conversation.

Please provide:
1. Which spec? (e.g., 001)
2. What issue was fixed? (brief description)
3. What was the root cause?
4. What was changed to fix it?
```

Then proceed with user-provided details instead of inferred ones.

## Important Notes

- **No arguments needed** - Command infers everything from context
- **Works with any workflow** - User can fix issues however they want (plan mode, normal chat, etc.)
- **User confirms details** - Always show summary and wait for "yes" before documenting
- **Atomic operation** - Documentation + commit happen together (no partial state)
- **Maintains history** - Creates permanent record in refinements.md with commit SHA
- **Git clean check** - Won't document if there are no changes to commit
- **File creation on demand** - refinements.md is only created when first refinement is documented
- **Clean separation** - tasks.md stays focused on implementation, refinements.md tracks fixes

## Examples

### Example 1: Simple Bug Fix

```
[User and Claude fix an API key validation issue together]

User: /document-refinement

Claude: 📋 Refinement Summary (Spec 001, R001)

**Issue:**
API key validation rejecting valid modern OpenAI keys

**Root Cause:**
Validation pattern in config.ts only accepted legacy 51-54 char keys,
but modern OpenAI keys use sk-proj- prefix and are 56-156+ chars

**Solution:**
Updated API_KEY_PATTERN regex to accept both legacy and modern formats

**Files Modified:**
- frontend/src/services/ai-chat/config.ts: Updated regex pattern
- frontend/src/services/ai-chat/index.ts: Updated docstring
- frontend/src/utils/error-messages.ts: Updated error message

Is this correct? (yes/edit/cancel)

User: yes

Claude: [Creates specs/001-minimum-playable/refinements.md, commits, asks about push]

✅ Refinement R001 documented and committed
Commit: abc123f
Documented In: specs/001-minimum-playable/refinements.md
```

### Example 2: User Corrects Details

```
User: /document-refinement

Claude: [Shows inferred summary]

User: edit

Claude: Which details need editing?

User: cause

Claude: What was the actual root cause?

User: The regex didn't account for hyphens in the key body

Claude: [Shows updated summary]

User: yes

Claude: [Documents in refinements.md and commits]
```

### Example 3: No Changes to Document

```
User: /document-refinement

Claude: ❌ No uncommitted changes found.

Please make refinement fixes first, then run /document-refinement.
```

### Example 4: Adding to Existing Refinements

```
[User already has refinements.md with R001, R002]
[User fixes another issue]

User: /document-refinement

Claude: 📋 Refinement Summary (Spec 003, R003)

**Issue:**
...

Claude: [Appends R003 to existing specs/003-frontend-backend-integration/refinements.md]

✅ Refinement R003 documented and committed
Documented In: specs/003-frontend-backend-integration/refinements.md
```
