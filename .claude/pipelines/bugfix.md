# Pipeline: Bugfix

Systematic bug investigation and fix.

## Steps

### 0. RESEARCH (researcher, Opus) | GATE: none
- **Input**: bug report, error logs
- **Actions**:
  - Read affected files + their imports and tests
  - `git log --oneline -5 -- <affected_files>` — did a recent change cause this?
  - Check `tasks/lessons.md` — was this bug pattern seen before?
  - Check `PROJECT_SPEC.md` — dependencies or constraints relevant to the bug
  - Search Engram globally for similar bugs
- **Output**: research context for reproduction and diagnosis
- **Also**: classify risk level per `risk-classification.md` (drives verification gates and review depth)

### 1. REPRODUCE (implementer, Sonnet) | GATE: none
- **Input**: bug report, error logs, research context, steps to reproduce
- **Output**: minimal reproduction case, confirmed error
- **Uses**: debug skill step 1

### 2. DIAGNOSE (researcher, Opus) | GATE: none
- **Input**: reproduction case, affected files, research context
- **Output**: root cause analysis, affected code paths
- **Uses**: debug skill steps 2-3

### 3. FIX (implementer, Sonnet) | GATE: typecheck
- **Input**: root cause + affected files
- **Output**: minimal fix (prefer root cause over symptom)
- **Constraint**: change as few files as possible

### 4. TEST (test-engineer, Sonnet) | GATE: tests_pass
- **Input**: fix diff + original reproduction case
- **Output**: regression test that fails without fix, passes with fix
- **Also**: run full test suite for affected modules

### 5. COMMIT (implementer, Sonnet) | GATE: none
- **Input**: tested fix
- **Output**: `fix(scope): description`
- **Also**: update lessons.md if this bug pattern is new
