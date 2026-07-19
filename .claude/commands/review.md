You are performing a change review (Ian Bull: review CHANGES, not CODE).

## Usage
`/review` — review uncommitted changes
`/review [branch]` — review branch vs main

## Process

### Step 0: Change Size Check
```bash
git diff --stat | tail -1
```
- If > 400 lines changed: warn "Large change — consider splitting into smaller PRs for safer review"
- If > 1000 lines: ask user to split before proceeding with review

1. **Collect diff**: `git diff` for uncommitted, or `git diff main...[branch]` for branch
2. **Launch reviewer agent** in isolated subagent context
3. **For each changed file**, reviewer evaluates:

### Intent
- Does this change match the stated task?
- Any unrelated changes mixed in?
- Is scope appropriate?

### Impact
- What behavior changed?
- Who/what is affected downstream?
- Blast radius: 1 module / cross-cutting / public API?

### Failure Modes
- What could go wrong in production?
- Error paths handled?
- Security implications?

### Rollback
- Can this be reverted with `git revert`?
- Irreversible changes? (DB migrations, data deletion)
- Feature flag available?

## Output
```
## Review: [description]
### Verdict: ✅ PASS | ⚠️ NEEDS REVIEW | ❌ BLOCKED
### Intent: MATCH / MISMATCH
### Impact: LOW / MEDIUM / HIGH
### Failure Modes: NONE / CONCERNS
### Rollback: SAFE / CAUTION / RISKY
### Action Items: [specific fixes needed]
```
