You are safely rolling back the last committed change with full verification.

## Process

### Step 1: Identify what to rollback
```bash
git log --oneline -5
```
Show the last 5 commits. Ask user: which commit(s) to revert?
Show the diff of the target: `git show [hash] --stat`

### Step 2: Check for dependencies
Before reverting:
- Are there commits AFTER the target that depend on it?
- Are there database migrations that need a down migration first?
- Are there irreversible side effects (emails sent, webhooks fired, external state)?

### Step 3: Create revert commit
```bash
git revert [hash] --no-commit
```
Review the revert diff with `git diff --staged`.
Run tests. If tests pass: commit the revert. If fail: investigate first.

### Step 4: Verify
Run full test suite for affected modules.
Check that the specific behavior that was introduced is now gone.

### Step 5: Log
Add entry to `brain/01-daily/[date].md`:
- What was reverted and why
- Was it a clean revert or did it require manual fixes?
- What caused the original issue?

## Safety Rules
- NEVER use `git reset --hard` (destroys history, can't be undone)
- ALWAYS use `git revert` (creates new commit, history preserved)
- If migration rollback needed: run down migration BEFORE reverting code
- If unsure about scope: ask before proceeding
