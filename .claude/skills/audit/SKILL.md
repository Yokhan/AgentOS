---
name: audit
description: "Strict architectural and code audit with multi-lens synthesis. Trigger on: аудит, audit, строгий аудит, проверь код строго, арх аудит, code review strict, lens synthesis, проверь архитектуру."
---

# Multi-Lens Audit Skill

Deep audit that analyzes code/architecture through 5 independent lenses, then synthesizes findings into a unified verdict. Replaces manual "сделай строгий арх и код аудит через синтез линз".

## When to use
- After completing a feature or significant change
- Before releases or major refactors
- When reviewing an unfamiliar codebase
- When user says "аудит", "audit", "проверь строго"

## Input
The user specifies ONE of:
1. **Project** — audit entire project or directory (default: current working directory)
2. **Files** — audit specific files or a PR diff
3. **Last work** — audit what was done in current/last task (reads `tasks/current.md` for changed files)

If no target specified, ask: "Что аудитим? Весь проект, конкретные файлы, или последнюю работу?"

## Process

### Phase 1: Scope & Context (read-only)
1. Determine target files/directories
2. If "last work" — read `tasks/current.md` for changed files list
3. Read all target files to understand the codebase
4. Count total lines/files to estimate audit depth

### Phase 2: Multi-Lens Analysis
Launch **up to 3 parallel Explore agents** (model: opus), each covering 1-2 lenses.

#### Lens 1: Architecture
- Module boundaries and dependency direction (no circular deps)
- Layer violations (does UI call DB directly?)
- Coupling analysis: connascence types, fan-in/fan-out
- Single Responsibility at module level
- Is complexity justified by requirements?

#### Lens 2: Code Quality
- Anti-patterns from `domain-software-review` checklist (god objects, deep nesting, primitive obsession, etc.)
- Naming clarity and consistency
- Function size (<20 lines), file size (<375 lines)
- Error handling: fail-fast, no swallowed exceptions
- Dead code, commented-out code, TODOs without tickets
- DRY violations vs premature abstraction

#### Lens 3: Security
- OWASP Top 10 check (injection, XSS, CSRF, auth issues)
- Hardcoded secrets or credentials
- Input validation at boundaries
- Dependency vulnerabilities (if package manager available)
- Principle of least privilege

#### Lens 4: Performance & Scalability
- O(n^2) or worse algorithms where better exists
- N+1 queries, missing indexes
- Unbounded collections, memory leaks
- Caching opportunities
- Blocking operations in async context

#### Lens 5: Developer Experience (DX)
- Is the code readable by a new team member?
- Are errors actionable (clear messages, not stack traces)?
- Is the API intuitive or surprising?
- Test quality: behavior-driven, not implementation-coupled
- Documentation: sufficient but not excessive

### Phase 3: Synthesis
After all lenses complete, synthesize:

1. **Cross-lens patterns** — issues that appear in 2+ lenses (e.g., god object = architecture + quality + DX problem)
2. **Risk matrix** — severity (Critical/High/Medium/Low) x likelihood (Certain/Likely/Possible/Unlikely)
3. **Root causes** — why these issues exist (rushed deadline? missing knowledge? tech debt?)
4. **Priority ranking** — order fixes by: impact x effort (quick wins first)

### Phase 4: Verdict & Report
Rate each lens: PASS / WARN / FAIL

```
## Audit Report — [target] — [date]

### Verdict: [PASS / WARN / FAIL]

| Lens           | Rating | Findings |
|----------------|--------|----------|
| Architecture   | ...    | N issues |
| Code Quality   | ...    | N issues |
| Security       | ...    | N issues |
| Performance    | ...    | N issues |
| DX             | ...    | N issues |

### Critical (fix now)
1. ...

### High (fix this sprint)
1. ...

### Medium (fix this quarter)
1. ...

### Low (nice to have)
1. ...

### Cross-Lens Patterns
- [pattern]: appears in [lens1, lens2] — root cause: [why]

### Quick Wins (high impact, low effort)
1. ...
```

Save report to `brain/03-knowledge/audits/audit-YYYY-MM-DD.md`.
If Critical findings — also add to `tasks/current.md` as next action items.
If new anti-patterns discovered — add to `tasks/lessons.md`.

## Notes
- NEVER report "looks good" without evidence. Every PASS needs at least one specific observation.
- If a lens has no findings, explicitly state what was checked and why it passed.
- Use evidence levels from `domain-software-review` (A/B/C) when citing anti-patterns.
- For "last work" audits, also check the diff quality: are commits atomic? Messages clear?
