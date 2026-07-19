---
name: decompose
description: "Breaks XL tasks (>15 files or architecture changes) into 3-5 M-sized sub-tasks with dependency graph. Called by /implement when task is too large."
---

# XL Task Decomposition

## When to Use
- Task classified as XL (>15 files, architecture change, migration)
- `/implement` Phase 1 detects XL scope
- `/sprint` encounters a task too large for single loop iteration

## Process

### 1. Analyze Scope
- List ALL files that would need to change
- Identify independent components (types, backend, frontend, tests, docs)
- Map module boundaries that would be crossed

### 2. Identify Dependencies
- Which changes depend on other changes?
- A→B means B needs A completed first
- Independent changes can run in parallel

### 3. Group into M-Sized Tasks
Each sub-task should be:
- 3-7 files
- ≤150 lines changed
- Self-contained (can be committed independently)
- Testable independently

### 4. Write Decomposition Plan

Write to `tasks/current.md`:

```markdown
## Decomposed: [original task] (XL → Nx M)

### Task A: [name] [NO DEPS]
- Files: [list]
- Estimate: S/M
- Tests: [what to test]
- Commit: [suggested commit message]

### Task B: [name] [DEPENDS: A]
- Files: [list]
- Estimate: S/M
- Tests: [what to test]
- Commit: [suggested commit message]

### Task C: [name] [DEPENDS: A, parallel with B]
- Files: [list]
- Estimate: S/M
- Tests: [what to test]
- Commit: [suggested commit message]

### Dependency Graph
A → B, C (parallel) → D → E
```

### 5. Present to User
Show the decomposition plan. Wait for approval before executing.

### 6. Execute
- Tasks with no deps → execute first
- Independent tasks → parallel if possible (see implement.md parallel execution)
- Each task follows its own ceremony level (M = OODA + 2 gates)
- Commit after each task (not at the end)

## Common Decomposition Patterns

### New Feature (frontend + backend)
```
A: Shared types/interfaces [NO DEPS]
B: Backend service + adapter [DEPENDS: A]
C: Frontend component + store [DEPENDS: A]
D: Integration wiring (routes, exports) [DEPENDS: B, C]
E: Tests + docs [DEPENDS: D]
```

### Migration
```
A: New schema/types alongside old [NO DEPS]
B: New implementation behind feature flag [DEPENDS: A]
C: Migration script (data) [DEPENDS: A]
D: Switch traffic to new + remove old [DEPENDS: B, C]
E: Cleanup old code + flag [DEPENDS: D]
```

### Refactoring
```
A: Characterization tests for current behavior [NO DEPS]
B: Extract shared module [DEPENDS: A]
C: Migrate consumers one-by-one [DEPENDS: B]
D: Remove old code [DEPENDS: C]
E: Update docs [DEPENDS: D]
```

## Anti-Patterns
- Don't decompose into tasks smaller than S — overhead exceeds benefit
- Don't create circular dependencies between tasks
- Don't defer ALL tests to the last task — test each task independently
- Don't decompose what's actually M-sized — decomposition itself costs time
