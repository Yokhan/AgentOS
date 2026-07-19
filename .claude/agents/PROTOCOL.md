---
name: protocol
description: Shared protocol loaded by ALL agents — routing, verification, memory, handoff
type: agent
---

# Agent Protocol v3.2 — Shared by ALL agents

> This file is loaded alongside agent definitions. Do NOT duplicate this content in agent files.

## Task Routing (FIRST THING every agent does)

Before starting work: `get_context(keywords="<your task keywords>")` → receive rules + context.
If task changes mid-work: `switch_context(keywords="<new keywords>")`.
Fallback (no MCP): `bash scripts/route-task.sh "<keywords>"` → Read listed files.
Rules are in `.claude/library/`, NOT pre-loaded. Load only what you need.

## Pre-Implementation Verification (ALL agents, MANDATORY for M+ tasks)

Before writing ANY code, complete this checklist:
- [ ] Read all affected files AND their neighbors and tests
- [ ] Stated user's goal in own words
- [ ] Considered at least 2 approaches and chose one with reasoning
- [ ] Identified the riskiest part of the plan
- [ ] Checked lessons.md and tool-registry for prior solutions

Skipping this checklist is a system failure.

### Commitment Bias Check (at 50% implementation)
Ask: "If I started fresh right now, would I choose this exact approach?"
- YES → continue
- NO or PROBABLY NOT → stop, reassess. Sunk cost is not a reason to continue.

See also: `.claude/library/process/self-verification.md` — Doubt Protocol, Sunk Cost Test.

## Claude-Specific Bias Corrections

> Claude tends to rush to implementation without deep thinking. These gates counteract that.

### Anti-Rush Protocol
If you catch yourself writing code before completing research:
1. STOP immediately
2. Delete what you wrote
3. Return to research phase
4. Code written on wrong assumptions has zero value — starting over is free

### Sycophancy Circuit Breaker
When your reaction to user feedback is "you're right!":
1. That means you ALREADY KNEW but didn't surface it
2. Log to tasks/lessons.md: what the flaw was, why you didn't catch it
3. Next time: same category of flaw must be caught by YOUR verification

## Codex-Specific Context Gates

> Codex is a strong engineer but needs explicit user intent and richer context.

### Success Criteria Protocol (MANDATORY)
Before implementing, state explicitly:
1. **"User wants:** [goal in user's own terms]"
2. **"Success means:** [measurable outcome — what changes, what works after]"
3. **"I will verify by:** [specific check — test, manual verification, output comparison]"

If you cannot state all three clearly, ASK the user before writing code.

### Context Loading (at task start)
Read these files before touching code:
1. `PROJECT_SPEC.md` — stack, dependencies
2. `tasks/current.md` — active work, handoff from previous session
3. `tasks/lessons.md` — past mistakes to avoid (ALL entries)
4. `_reference/tool-registry.md` — existing utilities (SEARCH before creating new)

## Subagent Context Discipline

When orchestrator launches you as a subagent:
- You receive: task description + file paths + findings summary from orchestrator
- Call `get_context(keywords="<your subtask>")` to load YOUR rules
- Do NOT re-read files that orchestrator already summarized in the prompt
- Stay focused on your specific subtask, don't expand scope

## Memory Protocol

When saving to Engram: use `topic_key="agent:{agent_name}:{category}"`.
Shared observations (useful across agents): `topic_key="shared:{category}"`.

Before editing a file: extract module name from path, `mem_search("{module}")` for related bugs/decisions/patterns.
When reading: search own namespace first, then shared. Search globally (omit project param) for cross-project insights.
If Engram unavailable: use file fallback (tasks/lessons.md + brain/).

**PROACTIVE saves** — after every decision, bug fix, discovery, convention. Don't wait to be asked.

## Handoff Output

When passing work to another agent, write to `tasks/current.md` under `## Agent Handoff`:
- **From**: {this_agent} → **To**: {next_agent}
- **Task**: one-line summary
- **Findings**: key discoveries
- **Files**: affected paths
- **Constraints**: what must not break
- **Confidence**: HIGH/MEDIUM/LOW
- **Blockers**: if any

## Context Budget

Stay within your tool call budget. If approaching limit:
1. Summarize current state to tasks/current.md
2. Save key findings to Engram
3. Stop gracefully with handoff context

| Agent | Budget |
|-------|--------|
| implementer | ~50 calls |
| researcher, devops, profiler | ~20 calls |
| reviewer, documenter | ~15 calls |
| test-engineer, simplifier, writer, security-auditor | ~25-30 calls |

## Metrics

On task completion, log metrics via agent-metrics skill (`.claude/skills/agent-metrics/SKILL.md`).

## Note

Old "Subagent Context Discipline" section removed (contradicted lines 11-18 above).
Subagent routing discipline is defined in the "Subagent Context Discipline" section at the top of this file.
