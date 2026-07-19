You are creating a git commit with an optional pull request.

## Process

### Step 1: Check State
Run `git status` and `git diff --staged` to understand what's being committed.

### Step 2: 11-Point Deploy Check
Before committing, verify all 11 points:
1. TypeScript/type check passes (`npx tsc --noEmit` / `mypy` / `cargo check`)
2. Linter passes (no errors)
3. All tests pass (unit + integration for affected modules)
4. No secrets committed (check for API keys, tokens, .env content)
5. Environment variables documented in `.env.example`
6. Architecture boundaries respected (no deep imports bypassing index)
7. API contracts updated in `docs/API_CONTRACTS.md` (if API changed)
8. Database migrations included and tested (if schema changed)
9. Rollback plan documented for breaking changes
10. CHANGELOG or brain/04-decisions/ updated for significant changes
11. On a feature branch (not committing directly to main/master)

If any fail, fix them first.

### Step 3: Commit
- Stage relevant files (specific files, not `git add .`)
- Write a conventional commit message: `type: description`
  - feat: new feature
  - fix: bug fix
  - refactor: code change without behavior change
  - docs: documentation
  - test: tests
  - chore: maintenance
- Keep subject < 72 chars
- Add body explaining WHY if non-obvious

### Step 4: Push & PR (if requested)
If user wants a PR:
- Push to remote with `-u` flag
- Create PR using `gh pr create`
- Title: same as commit convention
- Body: summary + test plan

### Step 5: Log
Update session log in brain/01-daily/ with the commit.
