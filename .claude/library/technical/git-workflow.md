# Git Workflow Rules

## Branches
- `main` — protected, always deployable. Never edit directly.
- `feat/description` — new features
- `fix/description` — bug fixes
- `refactor/description` — refactoring without behavior change
- `docs/description` — documentation only

## Commits
- Conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Message format: `type: short description` (< 72 chars)
- Body (optional): explain WHY, not WHAT
- One logical change per commit

## Before Commit
1. Run typecheck
2. Run linter
3. Run tests for affected modules
4. All must pass. No exceptions.

## Pull Requests
- Title: same as commit convention
- Body: what changed, why, how to verify
- Link related issues
- Keep PRs small (< 400 lines changed)

## Merge Strategy
- `git pull --rebase` by default (keeps history linear).
- Merge commits only for feature branch → main merges.
- Never rebase published commits (pushed to remote).

## Dependency Management
- Always commit lock files (package-lock.json, Pipfile.lock, go.sum, Cargo.lock).
- Pin exact versions for production dependencies.
- Run `npm audit` / `pip audit` / `cargo audit` in CI.
- Never use `*` or `latest` as version.

## Feature Flags (Deploy != Release)
- Deploying code to production and releasing it to users are SEPARATE operations.
- Use feature flags to deploy code behind a toggle: `if (flags.newCheckout) { ... }`
- Benefits: deploy anytime (even Friday), test in production with internal users, instant rollback without redeploy.
- Flag lifecycle: create -> develop behind flag -> test -> gradual rollout -> 100% -> remove flag and dead code.
- Clean up flags within 2 weeks of full rollout. Stale flags are tech debt that compounds.
- Flag naming: `enable_[feature]_[date]` — the date signals when to clean up.

## Commit Message Body
- Subject line: WHAT changed (< 72 chars, conventional commit format).
- Body (after blank line): WHY this change was made. What problem does it solve? What was the alternative?
- Link to issue/task: `Closes #123` or `Refs PROJ-456` — connects code to intent.
- If the commit is non-obvious, explain HOW (approach taken, trade-offs made).
- Future-you reading `git log` should understand the reasoning without opening the PR.

## Branch Lifetime Limit
- Feature branches must merge within **2 days** or be rebased onto main.
- Long-lived branches = merge hell. Conflicts grow exponentially with time and team size.
- If a feature takes >2 days: break it into smaller increments, each behind a feature flag.
- Stale branches (>1 week with no activity): delete or rebase. They will only get harder to merge.
- Trunk-based development with short-lived branches reduces merge conflicts by 80%+.

## Stacked PRs for Large Changes
- When a change is too large for one PR (>400 lines) but logically connected:
- Split into a chain: PR1 (base) -> PR2 (builds on PR1) -> PR3 (builds on PR2).
- Each PR is reviewable independently, has its own tests, and can be merged sequentially.
- Label stacked PRs: `[1/3]`, `[2/3]`, `[3/3]` in PR title. Link to the chain in description.
- Rebase the stack when the base PR changes. Tools: `git-town`, `ghstack`, or manual rebase.

## Git Bisect Awareness
- Every commit on main must be BUILDABLE and pass tests. No "WIP" or "fix later" commits on main.
- Why: `git bisect` finds the commit that introduced a bug by binary search. Broken commits break bisect.
- Squash messy WIP commits before merging to main. The feature branch can be messy; main cannot.
- This also means: no "fix lint" or "fix tests" commits after the fact. Get it right in the feature branch.

## Cherry-Pick Policy
- Cherry-pick ONLY for hotfixes to production: critical bug in prod, fix is on main, need it on release branch.
- Never cherry-pick features or non-critical changes — they bypass the normal review/test flow.
- After cherry-picking: verify the fix exists on BOTH branches. Orphaned cherry-picks cause regression when branches merge.
- Document every cherry-pick: which commit, why, from where to where.

## Never
- Force push to main
- Commit secrets, API keys, .env files
- Skip pre-commit hooks
- Amend published commits
