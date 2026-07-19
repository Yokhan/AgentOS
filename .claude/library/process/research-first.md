# Research-First Protocol — Read Before You Write

## The Rule
Before ANY task that modifies code or content, the agent MUST research the context.
Writing code without reading context = inventing problems that don't exist + missing problems that do.

## Mandatory Research Checklist (BEFORE any code change)

### 1. Read affected files + neighbors
- Open every file you plan to modify
- Read files that import from / are imported by the target files
- Read colocated tests (module.test.ts, test_module.py)

### 2. Check recent history
```
git log --oneline -5 -- <affected_files>
```
Why: someone may have just changed the same code. Avoid conflicts and redundant work.

### 3. Check project memory
- `tasks/current.md` — is there active work that overlaps?
- `tasks/lessons.md` — has this exact problem been solved (or failed) before?
- `PROJECT_SPEC.md` — does the project have dependencies or constraints that affect this?

### 4. Check tool registry
- `_reference/tool-registry.md` — does a utility/component already exist for what you're about to build?
- If found → REUSE it. If close match → extend it. Don't write new code for solved problems.
- See `.claude/library/technical/atomic-reuse.md` for full protocol.

### 5. Check for cross-project impact
- If the change affects shared code, APIs, or exports: check who consumes them
- If `ecosystem.md` exists: check for downstream dependencies

### 6. State findings BEFORE coding
Output a brief research summary:
```
RESEARCH:
- Read: [files read]
- Recent changes: [relevant git log entries]
- Lessons: [relevant entries from lessons.md, or "none applicable"]
- Risks: [what could go wrong]
- Approach: [chosen approach and why]
```

## Research Cache (`tasks/.research-cache.md`)

Before running fresh research, check the cache — previous findings may still be valid.
After completing research, append findings to the cache with date, files, discoveries, status.

- The cache survives compaction (pre-compact hook references it)
- UPDATE existing entries when revisiting the same topic — don't create duplicates
- Mark entries `stale` when the related work is done and committed
- Delete stale entries periodically to keep the cache useful

## When to Skip (XS tasks only)
- Single-file typo fix (≤5 characters)
- Config value change with no logic impact
- Comment or documentation-only edit

Even for XS: still read the file before editing. Never edit blind.

## Why This Matters

Without research:
- 40% of "fixes" break something else (coupling they didn't see)
- 30% of new code duplicates existing utilities
- Past lessons get ignored, same mistakes repeat
- Cross-project breakage goes undetected

With research:
- Agent works WITH the codebase, not against it
- Existing solutions get reused
- Past mistakes don't repeat
- Changes fit the established patterns
