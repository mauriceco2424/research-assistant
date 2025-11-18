---
description: Run git add, commit with properly formatted message, and push to remote repository
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Overview

This command automates the git workflow: staging changes, creating a well-formatted commit message, and pushing to the remote repository. It enforces the project's commit message standards to maintain consistency and traceability.

## Commit Message Format

All commits in this project **MUST** follow this format:

```
<Title>: <Brief description> (<Task IDs if applicable>, <Phase/User Story>)

<Detailed explanation organized by functional area>

Files Modified:
- <file>: <description of changes>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Format Guidelines

#### Title Line
- Start with an **action verb** (capitalize first letter):
  - `Implement` - New feature, solution, or approach (use this for unconfirmed fixes)
  - `Fix` - Bug fix confirmed working by user (ONLY after user tests and confirms)
  - `Add` - Adding new files, dependencies, or minor features
  - `Update` - Modifying existing functionality
  - `Refactor` - Code restructuring without changing behavior
  - `Polish` - UI/UX improvements, styling, minor enhancements
  - `Remove` - Deleting code or features
  - `Optimize` - Performance improvements
  - `Refine` - Post-implementation fixes for completed specs (use with spec ID)
- Follow with a brief, specific description (not generic like "updates" or "changes")
- Include task IDs in parentheses if applicable: `(T096-T102, Phase 6, User Story 4)` or spec/refinement IDs: `(Spec 001, R002)`
- Keep title under 100 characters

**CRITICAL RULE - Never claim something is "fixed" until user confirms:**
- Use `Implement:` for solutions that haven't been tested by the user yet
- Only use `Fix:` AFTER the user has tested and confirmed the solution works
- If user reports an issue, implement a solution, then ask them to test with `/start` before calling it "fixed"

**Examples - CORRECT (before user testing):**
```
Implement URL-safe Base64 handling for PoB export codes
Implement OpenAI API key validation for modern key formats
Refine: Add diagnostic logging to parser (Spec 001, R001)
```

**Examples - CORRECT (after user confirms):**
```
Fix PoB parsing with URL-safe Base64 decoding
Fix API key validation rejecting modern OpenAI formats
Implement version detection and API error retry (T109, T111, T113, Phase 7)
```

**Examples - WRONG (claiming fixed before testing):**
```
Fix PoB parsing bug [DON'T - not tested yet]
Fixed API key validation [DON'T - user hasn't confirmed]
```

#### Body Section
- Organize by **functional area** or **component** (use bold headers like `**Frontend:**` or `**Parser:**`)
- For task-based work, list each task ID with its implementation details
- Explain the **why** and **what**, not just the **how**
- Use bullet points for clarity
- Reference specific functions, files, or components when relevant

**Example:**
```
**Error Handling (T103-T105)**
- T103: Added input validation with clear error messages for invalid PoB codes
- T104: Implemented graceful degradation when API key is missing
- T105: Added error boundaries for chat component failures

**UI Polish (T106-T108)**
- T106: Improved layout responsiveness on mobile devices
- T107: Enhanced loading states with skeleton screens
- T108: Standardized button styles across components
```

#### Files Modified Section
- **Always** include this section
- List each modified file with a description of what changed
- Use relative paths from project root
- Mark NEW files with `NEW:` prefix
- Group related files together

**Example:**
```
Files Modified:
- frontend/src/services/pob-parser/decode.ts: Enhanced error handling for malformed input
- frontend/src/components/Chat/ChatInterface.tsx: Added loading states and error boundaries
- frontend/src/utils/validation.ts: Implemented input validation helpers
- NEW: frontend/src/components/ErrorBoundary.tsx: Created reusable error boundary component
```

#### Footer
**Always** include these two lines exactly as shown:
```
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

## Execution Flow

When the user runs `/git <optional description>`:

1. **Check Git Status**
   ```bash
   git status
   ```
   - If no changes to commit, inform user and exit
   - Show user which files will be staged

2. **Analyze Changes**
   ```bash
   git diff
   git diff --cached
   ```
   - Review all staged and unstaged changes
   - Identify the nature of changes (new feature, bug fix, refactor, etc.)

3. **Check Recent Commits** (for context and style consistency)
   ```bash
   git log -3 --oneline
   git log -1 --format='%B'
   ```
   - Review recent commit messages for style guidance
   - Check for task ID patterns if working in a task-based workflow

4. **Draft Commit Message**
   - Use the user's `$ARGUMENTS` as guidance (if provided)
   - Analyze the actual code changes to create accurate description
   - Follow the format guidelines above **strictly**
   - Choose the most appropriate action verb based on changes
   - Include task IDs if the changes are part of tracked tasks (check for tasks.md or similar)
   - Organize the body by functional area or component
   - List all modified files with descriptions
   - **Always** include the Claude Code attribution footer

5. **Stage All Changes**
   ```bash
   git add .
   ```

6. **Create Commit**
   Use a heredoc to ensure proper formatting:
   ```bash
   git commit -m "$(cat <<'EOF'
   [Your formatted commit message here]
   EOF
   )"
   ```

7. **Push to Remote**
   - Check if upstream is set for current branch:
     ```bash
     git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null
     ```
   - If upstream not set, push with `-u` flag:
     ```bash
     git push -u origin <current-branch>
     ```
   - Otherwise, push normally:
     ```bash
     git push
     ```

8. **Handle Failures**
   - **Pre-commit hook failures**:
     - If hooks modify files, check if safe to amend (verify authorship and not pushed)
     - If safe: amend commit with hook changes
     - If not safe: create new commit with hook fixes
   - **Push failures**:
     - If rejected due to remote changes: suggest `git pull --rebase`
     - If authentication fails: provide guidance on authentication setup
     - If other errors: show error and suggest next steps

9. **Report Success**
   - Show the commit SHA
   - Show the push result
   - Confirm branch and remote

## Important Notes

- **NEVER** skip pre-commit hooks (`--no-verify`) unless explicitly requested by user
- **NEVER** force push (`--force`) unless explicitly requested by user
- **NEVER** amend commits from other authors
- **NEVER** create empty commits
- **ALWAYS** ensure the commit message follows the format exactly
- **ALWAYS** include the Claude Code attribution footer
- The commit message quality is as important as the code changes themselves
- If you cannot determine appropriate task IDs or phase info, omit them (don't guess)

## Example Scenarios

### Scenario 1: Bug Fix
```
User: /git fixed the parser error with invalid base64

Title: Fix parser error handling for invalid base64 input

**Parser Service:**
- Enhanced base64 validation before decompression
- Added try-catch blocks with specific error messages
- Implemented fallback behavior for corrupted data

**Error Handling:**
- Created user-friendly error messages for common failure cases
- Added logging for debugging malformed input

Files Modified:
- frontend/src/services/pob-parser/decode.ts: Added base64 validation and error handling
- frontend/src/utils/validation.ts: Created base64 validation helper function

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Scenario 2: Feature Implementation
```
User: /git

Title: Implement chat history persistence (T087-T089, Phase 6)

**Session Storage (T087):**
- Implemented conversation history persistence using sessionStorage
- Added automatic save on each message exchange
- Created recovery mechanism for page refreshes

**State Management (T088):**
- Enhanced Zustand store with persistence middleware
- Added chat history hydration on app initialization

**UI Updates (T089):**
- Added visual indicator for restored conversations
- Implemented clear history button with confirmation

Files Modified:
- frontend/src/store/chatStore.ts: Added persistence middleware and hydration logic
- frontend/src/components/Chat/ChatInterface.tsx: Implemented history restoration UI
- frontend/src/utils/storage.ts: Created storage utility functions

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

## Validation Checklist

Before committing, verify:
- [ ] Title uses appropriate action verb and is specific
- [ ] Title includes task IDs if applicable
- [ ] Body is organized by functional area/component
- [ ] Body explains what changed and why
- [ ] All modified files are listed with descriptions
- [ ] Claude Code attribution footer is included
- [ ] No placeholder text like "[description]" remains
- [ ] Commit message is accurate to actual code changes
