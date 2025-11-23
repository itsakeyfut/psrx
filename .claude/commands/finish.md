---
description: Commit changes and create PR (keep under 100 lines)
allowed-tools: ["bash", "read", "grep"]
argument-hint: "[file1] [file2] ..."
---

Complete the implementation workflow:

**Steps:**

1. **Check current status:**
   ```bash
   git status
   git diff --stat
   ```

2. **Verify changes are under 100 lines:**
   ```bash
   git diff | wc -l
   ```
   Check the diff size. If over 100 lines, consider splitting into multiple PRs.

3. **MANDATORY: Verify and move to working branch:**

   **CRITICAL**: NEVER commit directly to `main` or `develop` branch!

   ```bash
   # Check current branch
   git branch --show-current
   ```

   **If currently on `main` or `develop`:**
   - STOP and create/switch to a feature branch first
   - Example: `git checkout -b feat/issue-XXX` or `git checkout existing-branch`

   **If already on a feature branch:**
   - Verify the branch name is correct
   - Proceed to next step

   **Branch naming convention:**
   - `feat/<description>` for features
   - `fix/<description>` for bug fixes
   - `refactor/<description>` for refactoring
   - `docs/<description>` for documentation

4. **Run quality checks:**
   ```bash
   cargo x fmt
   cargo x clippy
   cargo x test
   ```

5. **Stage and commit changes:**

   **File selection:**
   - If specific files were provided as arguments: `$ARGUMENTS`
     → Use: `git add $ARGUMENTS` (commit only specified files)
   - If no arguments were provided:
     → Use: `git add .` (commit all changed files)

   **Commit guidelines:**
   - Create logical, atomic commits
   - Follow conventional commits format (feat/fix/docs/refactor/test/chore)
   - Reference issue numbers with "Closes #XXX"
   - Example: `feat(gpu): implement VRAM transfer commands\n\nCloses #29`

6. **Push changes:**
   ```bash
   git push -u origin <branch-name>
   ```

7. **Create PR using gh command:**
   ```bash
   gh pr create --title "..." --body "..."
   ```

**PR Guidelines:**

**MANDATORY PR Body Limit: MAXIMUM 100 LINES**

- **Keep PR body concise** - MUST be under 100 lines
- Use clear, concise language
- Include only essential information:
  - Brief summary (2-4 sentences)
  - Key changes (3-5 bullet points)
  - Test plan (brief checklist)
  - "Closes #XXX" reference
- Avoid verbose descriptions, excessive formatting, or redundant information
- If more details are needed, add them as issue comments instead

**PR Title:**
- Follow conventional commits format
- Example: `feat(component): brief description`

Please proceed with these steps.
