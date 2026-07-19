---
name: hotfix
description: "Fast-track production fix. Skips ceremony, requires post-mortem. Use only for critical production issues."
---

# /hotfix [description]

Emergency production fix. Minimal ceremony, maximum speed.

## Constraints
- **Max 3 files** changed
- **Max 50 lines** changed
- If bigger → "This is not a hotfix. Use `/implement`."
- Post-mortem is **MANDATORY** — cannot be skipped

## Steps

### 1. ASSESS (≤2 min)
- Read error message/logs
- Identify root cause
- Confirm this is production-critical (not just a nuisance)

### 2. FIX
- Minimal change to resolve the issue
- ONE file preferred
- No refactoring, no cleanup, no "while I'm here" improvements
- If the fix reveals a deeper issue → fix the symptom NOW, file a task for the root cause

### 3. SMOKE TEST
- Test ONLY the broken path
- Verify the fix resolves the specific error
- Run typecheck (fast, catches obvious breaks)
- Skip full test suite — speed matters here

### 4. COMMIT
```
fix(scope): description [HOTFIX]

Emergency fix for [what was broken].
Root cause: [brief].
Follow-up needed: [full tests, review, prevention rule].
```

### 5. POST-MORTEM (MANDATORY)

Write to `tasks/current.md`:
```markdown
## Hotfix Post-Mortem — [date]
- **What broke**: [description]
- **User impact**: [who was affected, for how long]
- **Root cause**: [why it happened]
- **Fix applied**: [what was changed]
- **Follow-up needed**:
  - [ ] Full test suite run
  - [ ] Code review of the fix
  - [ ] Prevention rule for lessons.md
  - [ ] Root cause fix (if symptom-only fix applied)
```

Write to `tasks/lessons.md`:
```
### [DATE] — Hotfix: [title]
**Error**: [what broke]
**Root cause**: [why]
**Rule**: [prevention rule]
**Applies to**: [general/project-specific]
```

## When NOT to Use /hotfix
- Bug is annoying but not blocking users → use `/implement`
- Fix requires >3 files → use `/implement` with high priority
- Root cause is unclear → use debug skill first
- No production deployment exists → regular fix workflow
