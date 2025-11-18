---
description: Create a pull request from current feature branch to main branch
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Overview

This command creates a pull request (PR) from the current feature branch to the main branch. It automatically generates a PR description based on the spec files in the feature directory and provides the PR URL for review.

## Prerequisites

Before running this command, the user should have:
1. Completed their feature implementation
2. Run `/git` to commit and push all changes
3. Validated that the feature works as expected

## Execution Flow

When the user runs `/pr <optional title override>`:

1. **Verify Current Branch**
   ```bash
   git branch --show-current
   ```
   - Ensure not on `main` or `master` branch
   - If on main/master, error: "Cannot create PR from main branch. Switch to your feature branch first."
   - Extract feature name from branch (e.g., `003-auth` → `auth`)

2. **Check Branch Status**
   ```bash
   git status
   git log origin/main..HEAD --oneline
   ```
   - Verify all changes are committed (no uncommitted changes)
   - Verify branch has commits ahead of main
   - If uncommitted changes exist: "You have uncommitted changes. Run `/git` first to commit and push."
   - If no commits ahead: "Your branch has no new commits. Nothing to create PR for."

3. **Verify Branch is Pushed**
   ```bash
   git ls-remote --heads origin $(git branch --show-current)
   ```
   - If branch not on remote: Push it
     ```bash
     git push -u origin $(git branch --show-current)
     ```

4. **Find Spec Directory**
   - Extract spec ID from branch name (e.g., `002-backend-engine` → `002`)
   - Look for matching spec directory: `specs/002-*` or `specs/002`
   - If found, read spec.md for PR description context

5. **Generate PR Title**
   - If user provided `$ARGUMENTS`: Use as title
   - Otherwise, generate from:
     - Spec title (from spec.md if available)
     - Branch name as fallback
   - Format: `<Feature Name>: <Brief Description>`
   - Example: `Backend Engine: Implement core API and external integrations`

6. **Generate PR Description**
   Use this template:
   ```markdown
   ## Summary

   <2-3 bullet points describing what this PR adds, based on spec.md or changes>

   ## Changes

   <List key files/components modified, grouped by area>

   ## Testing

   - [ ] Feature works as specified
   - [ ] No regressions introduced
   - [ ] Code follows project conventions

   ## Spec Reference

   <Link to spec directory if exists, e.g., "See specs/002-backend-engine/ for full specification">

   ---

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   ```

7. **Identify Main Branch**
   - **IMPORTANT**: Always use `main` as the base branch for PRs in this project
   - Do NOT auto-detect using `git remote show origin | grep 'HEAD branch'` as this can be stale
   - The target is always `main` unless user explicitly specifies otherwise

8. **Create Pull Request**
   ```bash
   gh pr create --base <main-branch> --title "<title>" --body "$(cat <<'EOF'
   <generated description>
   EOF
   )"
   ```
   - Use GitHub CLI (`gh`) to create the PR
   - If `gh` not available, provide instructions for manual PR creation

9. **Handle Errors**
   - **gh not installed**: "GitHub CLI (gh) not found. Install it or create PR manually at: https://github.com/<repo>/compare/<main>...<branch>"
   - **Not authenticated**: "Run `gh auth login` to authenticate with GitHub"
   - **PR already exists**: Show existing PR URL

10. **Report Success**
    - Show PR URL
    - Show PR number
    - Remind user to request review
    - Example output:
      ```
      ✓ Pull Request created successfully!

      PR #12: Backend Engine: Implement core API and external integrations
      URL: https://github.com/user/repo/pull/12

      Next steps:
      1. Share PR URL with reviewer
      2. Wait for approval
      3. Run /merge after approval
      ```

## Important Notes

- **NEVER** create a PR to a branch other than main/master unless explicitly requested
- **ALWAYS** verify changes are committed and pushed before creating PR
- **ALWAYS** include the spec reference in the PR description if spec exists
- **ALWAYS** provide the PR URL at the end
- If spec.md exists, use it to create an accurate PR summary
- The PR description should help reviewers understand what changed and why

## Example Scenarios

### Scenario 1: Standard Feature PR
```
User: /pr

Branch: 002-backend-engine
Spec: specs/002-backend-engine/spec.md exists

Creates PR:
Title: Backend Engine: Implement core API infrastructure
Body: Summary from spec.md, list of changes, testing checklist
URL: https://github.com/user/repo/pull/12
```

### Scenario 2: Custom Title
```
User: /pr Add authentication support

Branch: 003-auth
Creates PR with user-provided title: "Add authentication support"
```

### Scenario 3: No GitHub CLI
```
User: /pr

gh not found, provides:
"Create PR manually at: https://github.com/user/repo/compare/main...003-auth"
Plus generated description for copy/paste
```

## Validation Checklist

Before creating PR:
- [ ] Not on main/master branch
- [ ] All changes committed (no uncommitted files)
- [ ] Branch pushed to remote
- [ ] PR title is descriptive
- [ ] PR description includes summary, changes, and testing sections
- [ ] Spec reference included if spec exists
