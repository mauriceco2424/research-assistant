---
description: Merge an approved pull request into the main branch
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Overview

This command merges an approved pull request into the main branch. It verifies the PR is ready to merge (approved, no conflicts) and performs the merge. If conflicts exist, it stops and instructs the user to run `/resolve-conflicts`.

## Prerequisites

Before running this command:
1. PR should be created (via `/pr`)
2. PR should be reviewed and approved
3. CI checks should pass (if configured)

## Execution Flow

When the user runs `/merge <optional PR number>`:

1. **Identify Current Branch and PR**
   ```bash
   git branch --show-current
   ```
   - Get current feature branch name

   If `$ARGUMENTS` contains a PR number, use that. Otherwise:
   ```bash
   gh pr list --head $(git branch --show-current) --json number,state,title
   ```
   - Find PR for current branch
   - If no PR found: "No PR found for branch. Run `/pr` first to create one."
   - If multiple PRs: List them and ask user to specify

2. **Check PR Status**
   ```bash
   gh pr view <PR_NUMBER> --json state,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup
   ```
   - **state**: Must be "OPEN" (not already merged or closed)
   - **mergeable**: Must be "MERGEABLE"
   - **reviewDecision**: Check if approved (APPROVED, REVIEW_REQUIRED, CHANGES_REQUESTED)
   - **statusCheckRollup**: Check CI status if configured

3. **Handle PR Status Issues**

   - **Already merged**:
     ```
     ✓ PR #12 is already merged!
     Your feature is in main. Consider running cleanup to delete the branch.
     ```

   - **Closed (not merged)**:
     ```
     ✗ PR #12 is closed but not merged.
     Reopen the PR on GitHub if you want to merge it.
     ```

   - **Not approved** (and approval required):
     ```
     ⏳ PR #12 is not yet approved.
     Current status: REVIEW_REQUIRED / CHANGES_REQUESTED

     Wait for reviewer approval before merging.
     ```

   - **CI checks failing**:
     ```
     ⏳ PR #12 has failing CI checks.

     Failed checks:
     - test: Build failed
     - lint: 3 errors found

     Fix the issues and push updates before merging.
     ```

   - **Has conflicts** (mergeable = "CONFLICTING"):
     ```
     ⚠️ PR #12 has merge conflicts with main.

     The main branch has changed since you created this PR.

     Run `/resolve-conflicts` to:
     1. See what conflicts exist
     2. Get AI-assisted resolution recommendations
     3. Resolve conflicts and update your PR

     After resolving, run `/merge` again.
     ```

4. **Confirm Merge Details**
   ```bash
   gh pr view <PR_NUMBER> --json title,additions,deletions,changedFiles
   ```
   Show user:
   ```
   Ready to merge PR #12: <Title>

   Changes: +<additions> -<deletions> across <changedFiles> files
   Base: main ← <feature-branch>

   Proceed with merge? (yes/no)
   ```

   Wait for user confirmation.

5. **Perform Merge**
   ```bash
   gh pr merge <PR_NUMBER> --merge --delete-branch
   ```
   - Uses merge commit (not squash or rebase) to preserve history
   - `--delete-branch` removes the feature branch after merge

   **Alternative merge strategies** (if user specifies):
   - `--squash`: Squash all commits into one
   - `--rebase`: Rebase commits onto main

6. **Handle Merge Errors**
   - **Merge blocked by branch protection**:
     ```
     ✗ Merge blocked by branch protection rules.

     Required conditions not met:
     - Required reviewers: Need 1 more approval
     - Status checks: CI must pass

     Resolve these requirements and try again.
     ```

   - **Unexpected conflict during merge**:
     ```
     ✗ Conflict detected during merge attempt.

     Run `/resolve-conflicts` to resolve and try again.
     ```

7. **Update Local Repository**
   After successful merge:
   ```bash
   git checkout main
   git pull origin main
   ```
   - Switch to main branch
   - Pull latest changes (includes the merge)

8. **Report Success**
   ```
   ✓ PR #12 merged successfully!

   Merged: <feature-branch> → main
   Commit: <merge-commit-sha>
   Branch deleted: <feature-branch>

   You are now on the main branch with latest changes.

   Next steps:
   - Start a new feature with /speckit.specify
   - Or continue working on another branch
   ```

## Important Notes

- **NEVER** force merge a PR with conflicts - always use `/resolve-conflicts` first
- **NEVER** merge without user confirmation
- **ALWAYS** check PR approval status before merging
- **ALWAYS** update local main after merge
- **ALWAYS** inform user if there are blocking issues
- The merge preserves the PR history and creates a merge commit
- Branch is automatically deleted after successful merge (can be restored if needed)

## Example Scenarios

### Scenario 1: Successful Merge
```
User: /merge

PR #12 is approved, no conflicts, CI passing.
User confirms merge.
PR merged, branch deleted, user switched to main.
```

### Scenario 2: Conflicts Present
```
User: /merge

PR #12 has conflicts.
Output:
⚠️ PR #12 has merge conflicts with main.
Run `/resolve-conflicts` to resolve and try again.
```

### Scenario 3: Awaiting Approval
```
User: /merge

PR #12 not yet approved.
Output:
⏳ PR #12 is not yet approved.
Wait for reviewer approval before merging.
```

### Scenario 4: Specific PR Number
```
User: /merge 15

Merges PR #15 regardless of current branch.
Useful when managing multiple PRs.
```

## Validation Checklist

Before merging:
- [ ] PR exists and is open
- [ ] PR is approved (or approval not required)
- [ ] CI checks pass (if configured)
- [ ] No merge conflicts
- [ ] User has confirmed the merge
