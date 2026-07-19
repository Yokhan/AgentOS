# Pipeline: Feature

Full lifecycle for adding a new feature.

## Steps

### 1. RESEARCH (researcher, Opus) | GATE: none
- **Input**: feature description, linked issue
- **Actions**:
  - Read all affected files + their imports and tests
  - `git log --oneline -5 -- <affected_files>` for recent changes
  - Check `tasks/lessons.md` for related past mistakes
  - Check `PROJECT_SPEC.md` for project constraints and dependencies
  - Search for existing utilities (Grep/Glob) before planning new ones
- **Output**: research summary — existing patterns, affected files, risks, reusable code, approach options
- **Also**: classify risk level per `risk-classification.md` (LOW/MEDIUM/HIGH/CRITICAL)
- **Budget**: ~20 tool calls

### 1.5. BRAINSTORM (researcher, Opus) | GATE: risk_threshold
- **Input**: research findings + feature description + risk classification
- **Trigger**: MEDIUM risk M+ (optional), HIGH (recommended), CRITICAL (mandatory)
- **Actions**:
  - Enumerate 3+ approaches with trade-off analysis (effort/risk/reversibility/extensibility)
  - Select recommended approach with justification
  - Document rejected approaches with reasons
- **Output**: brainstorm summary in `tasks/current.md` under `## Brainstorm`
- **Budget**: ~10 tool calls
- **Skip**: LOW risk, or size XS/S with MEDIUM risk

### 2. PLAN (implementer, Sonnet) | GATE: user_approval
- **Input**: research findings + feature description + brainstorm (if applicable)
- **Output**: written plan in `tasks/current.md` including:
  - Goal (1 sentence)
  - Complexity estimate (size, file count, line estimate)
  - File architecture (directory tree with purpose per file)
  - Implementation order with dependencies
  - File size check (nothing >375 lines)
  - Risks and mitigations
  - Test scenarios (happy path, edge cases, errors) per `plan-first.md` templates
- **Quality gate**: plan must pass Planning Quality Gate checklist (see `plan-first.md`)
- **Action**: present plan to user, wait for approval/annotations

### 3. IMPLEMENT (implementer, Sonnet) | GATE: typecheck
- **Input**: approved plan
- **Output**: code changes (batch write protocol: 3-4 files → typecheck)
- **Mid-build checkpoint**: after step 4 of implementation order

### 4. TEST (test-engineer, Sonnet) | GATE: tests_pass
- **Input**: changed files list
- **Output**: test files + coverage report
- **Minimum**: unit tests for new code, integration test for wiring

### 5. REVIEW (reviewer, Sonnet) | GATE: verdict:PASS
- **Input**: full diff (all changes from steps 3-4)
- **Output**: review verdict — PASS / NEEDS_REVIEW / BLOCKED
- **On NEEDS_REVIEW**: address feedback, re-submit for review

### 6. COMMIT (implementer, Sonnet) | GATE: none
- **Input**: reviewed, tested changes
- **Output**: conventional commit(s)
- **Format**: `feat(scope): description`
