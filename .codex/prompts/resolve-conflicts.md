---
description: Resolve merge conflicts with AI-assisted analysis and user-confirmed resolutions
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Overview

This command resolves merge conflicts between your feature branch and the main branch using AI-assisted analysis. It shows each conflict, explains what both sides were trying to accomplish, recommends a resolution, and **requires user confirmation before making any changes**. After resolving, it automatically retries the merge process.

## Prerequisites

This command is typically run after:
1. `/merge` failed due to conflicts, OR
2. You know your branch has conflicts with main

## Execution Flow

When the user runs `/resolve-conflicts`:

1. **Fetch Latest Main Branch**
   ```bash
   git fetch origin main
   ```
   - Ensure we have the latest main branch state

2. **Check Current Branch**
   ```bash
   git branch --show-current
   ```
   - Must not be on main/master
   - If on main: "Switch to your feature branch first."

3. **Start Rebase on Main**
   ```bash
   git rebase origin/main
   ```
   - This will either succeed (no conflicts) or stop at first conflict

4. **If No Conflicts**
   ```
   ✓ No conflicts found!

   Your branch is now up to date with main.
   Run `/merge` to complete the merge process.
   ```
   Skip to step 9 (push and retry merge).

5. **Identify All Conflicting Files**
   ```bash
   git diff --name-only --diff-filter=U
   ```
   - List all files with conflicts
   - Count total conflicts

6. **Analyze Each Conflict - SHOW TO USER BEFORE RESOLVING**

   **CRITICAL: Present all conflicts to user FIRST, get approval BEFORE making changes.**

   For each conflicting file:

   a. **Extract Conflict Markers**
      ```bash
      git diff <file>
      ```
      - Find `<<<<<<<`, `=======`, `>>>>>>>` markers
      - Extract both versions (yours and main's)

   b. **Analyze the Context**
      - Read surrounding code to understand purpose
      - Check git log for both branches to see commit messages
      ```bash
      git log --oneline HEAD -- <file>
      git log --oneline origin/main -- <file>
      ```

   c. **Generate Conflict Report**
      Present to user in this format:

      ```markdown
      ## Conflict Analysis Report

      Found **<N> conflicting file(s)** when rebasing on main:

      ---

      ### Conflict 1: <file_path>

      **Your version (<feature-branch>):**
      ```<language>
      <your code>
      ```

      **Main branch version:**
      ```<language>
      <main code>
      ```

      **What happened:**
      - Your branch: <explanation of what your changes were doing>
      - Main branch: <explanation of what main's changes were doing>

      **My recommendation:** <KEEP_YOURS | KEEP_THEIRS | COMBINE_BOTH | CUSTOM>
      ```<language>
      <recommended resolution>
      ```
      **Reasoning:** <why this resolution makes sense>
      **Confidence:** <High | Medium | Low>

      ---

      ### Conflict 2: <file_path>
      [... repeat for each conflict ...]

      ---

      ## Resolution Summary

      | # | File | Recommendation | Confidence |
      |---|------|----------------|------------|
      | 1 | <file1> | <recommendation> | High |
      | 2 | <file2> | <recommendation> | Medium |
      | ... | ... | ... | ... |

      ---

      ## Your Options

      - Type `yes` to apply ALL recommendations as shown above
      - Type `no` to abort (your branch remains unchanged)
      - Type `modify 1` to provide custom resolution for conflict 1
      - Type `modify 2,3` to modify conflicts 2 and 3
      - Type `skip` to skip conflict resolution and stay in conflicted state

      **Your choice:**
      ```

7. **Wait for User Confirmation**

   **DO NOT proceed until user responds.**

   Handle responses:

   - **User says `yes`**:
     Apply all recommended resolutions
     ```bash
     # For each file, write the recommended resolution
     # Then mark as resolved
     git add <file>
     ```

   - **User says `no`**:
     ```bash
     git rebase --abort
     ```
     ```
     ✗ Conflict resolution aborted.
     Your branch is unchanged.
     ```
     Exit command.

   - **User says `modify N`**:
     Ask for their custom resolution:
     ```
     Provide your custom resolution for Conflict N (<file>):

     You can:
     1. Paste the exact code you want
     2. Say "keep mine" to use your version
     3. Say "keep theirs" to use main's version
     4. Describe what you want combined

     Your resolution:
     ```
     Update the recommendation with user's input.
     Then ask for confirmation again.

   - **User says `skip`**:
     ```
     ⏸️ Conflict resolution paused.

     Your branch is in a conflicted state.
     Conflicting files:
     - <file1>
     - <file2>

     To continue manually:
     1. Edit files to resolve conflicts
     2. Run: git add <files>
     3. Run: git rebase --continue

     To abort: git rebase --abort
     ```

8. **Apply Resolutions** (after user confirms `yes`)

   For each conflict:
   ```bash
   # Write the resolved content to the file
   # Stage the resolved file
   git add <file>
   ```

   Continue the rebase:
   ```bash
   git rebase --continue
   ```

   - If more conflicts appear, repeat the analysis for the new conflicts
   - Keep going until rebase completes or user aborts

9. **Push Updated Branch**
   ```bash
   git push --force-with-lease origin <branch>
   ```
   - `--force-with-lease` is safer than `--force` (won't overwrite others' changes)
   - This updates the PR with resolved conflicts

10. **Retry Merge Process**
    ```bash
    gh pr view --json mergeable
    ```
    - Check if PR is now mergeable

    If mergeable:
    ```
    ✓ Conflicts resolved successfully!

    Your branch is now up to date with main.
    PR is ready to merge.

    Run `/merge` to complete the merge process.
    ```

    If still has issues:
    ```
    ✓ Conflicts resolved and branch updated.

    However, the PR still has issues:
    - <issue description>

    Address these before running `/merge`.
    ```

## Important Notes

- **CRITICAL: ALWAYS show conflict analysis to user and wait for confirmation BEFORE making any changes**
- **NEVER** automatically resolve conflicts without user approval
- **NEVER** force push without `--force-with-lease`
- **ALWAYS** explain what each side of the conflict was trying to do
- **ALWAYS** provide reasoning for recommendations
- **ALWAYS** give user options to modify, abort, or skip
- Low confidence recommendations should be highlighted for extra attention
- Complex conflicts (architectural changes, business logic) require careful human review
- The command can be run multiple times if new conflicts appear

## Resolution Strategies

When recommending resolutions, consider:

1. **KEEP_YOURS**: Your changes are additive and don't conflict with main's intent
2. **KEEP_THEIRS**: Main's changes supersede yours or your changes are no longer needed
3. **COMBINE_BOTH**: Both changes are independent and can coexist
4. **CUSTOM**: Changes conflict in intent and require careful merging

## Example Scenarios

### Scenario 1: Simple Additive Conflict
```
Your version adds: authToken field
Main version adds: cacheEnabled field

Recommendation: COMBINE_BOTH (High confidence)
Both are independent additions that don't interfere.
```

### Scenario 2: Overlapping Logic
```
Your version: Changed validation to require email
Main version: Changed validation to require phone

Recommendation: CUSTOM (Low confidence)
"Consider requiring both, or determine which is correct based on requirements."
```

### Scenario 3: Version Bump Conflict
```
Your version: 1.2.0
Main version: 1.1.1

Recommendation: KEEP_YOURS (High confidence)
Use higher version number to include both sets of changes.
```

## Validation Checklist

During conflict resolution:
- [ ] All conflicts identified and shown to user
- [ ] Each conflict has clear explanation of both sides
- [ ] Recommendations include reasoning and confidence level
- [ ] User explicitly confirmed before any changes made
- [ ] All resolutions applied correctly
- [ ] Rebase completed successfully
- [ ] Branch pushed with updates
- [ ] PR mergeable status checked
