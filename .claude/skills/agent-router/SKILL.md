---
name: agent-router
description: "Automatic agent selection based on task keywords and file context. Use when agent is not explicitly specified."
---

# Agent Router

Selects the best agent for a task based on signals from the request and affected files.

## When to Use
- User gives a task without specifying an agent
- `/implement` is called without agent override
- `/sprint` picks up a new task from queue

## Keyword Signals

| Keywords in task | Agent | Model |
|-----------------|-------|-------|
| fix, bug, error, broken, crash, regression | implementer + debug skill | Sonnet |
| review, check, audit (code review) | reviewer | Sonnet (Opus for deep) |
| research, investigate, find out, compare, evaluate | researcher | Opus |
| deploy, release, ship, infrastructure, CI/CD | devops | Sonnet |
| test, coverage, spec, TDD | test-engineer | Sonnet |
| simplify, refactor, cleanup, reduce complexity | simplifier | Sonnet |
| security, vulnerability, CVE, audit (security) | security-auditor | Opus |
| docs, readme, changelog, API docs | writer in informational mode | Opus |
| literary, marketing, informational, communication text | writer + writing-workflow | Opus |
| perf, slow, latency, profile, optimize | profiler | Sonnet |

## File Context Signals

When task references specific files, use file patterns to refine:

| File pattern | Agent adjustment |
|-------------|-----------------|
| `*.test.*`, `*.spec.*` | test-engineer |
| `Dockerfile`, `nginx.conf`, `.github/workflows/*`, `docker-compose*` | devops |
| `*.css`, `*.scss`, `*.tsx` with style changes | implementer + load domain-design rules |
| `auth/*`, `security/*`, `*.auth.*` | security-auditor (deep mode) |
| `shared/*`, `core/*` | reviewer (Opus, deep mode) — high blast radius |
| `*.md`, `docs/*` | writer in informational mode when content changes; documenter only for mechanical doc generation |
| `brain/*` | researcher (knowledge management) |

## Compound Signals

Some tasks need multiple agents in sequence:

| Combined signal | Pipeline |
|----------------|----------|
| "fix" + "security" | security-auditor → implementer |
| "add feature" + new module | researcher (patterns) → implementer |
| "refactor" + "shared/" | reviewer (assess) → simplifier (execute) |
| "deploy" + "fix" | implementer (hotfix) → devops (deploy) |

## Project-Specific Routing

If project has `project-*-implementer.md` agent overlays (created by `/setup-project`):
- Use project-specific agent instead of generic when file matches that stack
- Example: editing `.tsx` in a React project → `project-frontend-implementer` if it exists

## Fallback

If no clear signal matches: **implementer** (most common task type).

## Output Format

```
ROUTING:
- Task: [brief description]
- Signal: [keyword/file that triggered]
- Agent: [selected agent]
- Model: [sonnet/opus]
- Additional: [skills to load, rules to activate]
```
