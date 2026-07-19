---
name: codex-subagent-orchestration
description: "Use Codex subagents for parallel exploration, review, testing, docs research, design audit, and isolated implementation. Trigger when the task can be split across `.codex/agents` workers."
---

# Codex Subagent Orchestration

Use this skill when parallel work can reduce latency without creating edit conflicts.

## Default Pattern

1. Run `node scripts/codex-route-task.js "<user request>" --summary --write-state`.
2. Discover existing workflow artifacts first: Spec Kit, litkit, Kiro, AgentOS, `PROJECT_SPEC.md`, `tasks/current.md`, or project-local `project-*` skills.
3. If AgentOS is detected, treat it as the orchestrator and use Codex subagents only inside the assigned worker route.
4. Read the route's `fanout.status`, reason, candidates, and role profiles from `scripts/codex-agent-policy.js`.
5. For `required`, spawn the independent required lanes. For `recommended`, spawn without asking when a non-blocking lane materially improves speed, evidence, or context isolation. For `conditional`, spawn only after the independence gate passes. For `skip`, do not spawn.
6. Notify the user which agents started and why. Ask for narrow outputs with file references and verification steps.
7. Keep working on the parent critical path. Wait only when the next action needs a child result.
8. Consolidate in the parent thread. Parent performs edits unless an `implementer` task is isolated to non-overlapping files.

For prompt templates and the routing matrix, read `docs/CODEX_FANOUT_PATTERNS.md`.

## Safe Prompt

```text
Use Codex subagents with existing project artifacts.
First inspect whether this project has Spec Kit, litkit, Kiro, AgentOS, or project-local workflow docs.
Spawn pr_explorer, reviewer, and tester for read-only grounding.
Read-only means no file writes and no git restore/checkout/reset/clean, stash,
generated-artifact cleanup, or other shared-worktree state change. Report
unexpected changes to the parent; never repair or revert them.
Wait for all results. Parent agent performs edits unless exact [P] tasks with non-overlapping files are assigned.
```

## Guardrails

- Do not use subagents for XS tasks.
- Explicit user opt-out always wins.
- Do not spawn multiple write-capable agents on overlapping files.
- Before write fan-out, validate proposed `{ agent, files }` assignments with `validateWriteAssignments` from `scripts/codex-agent-policy.js`; any conflict blocks spawning.
- Treat `[P]` or equivalent project task metadata as the default signal for safe parallel work.
- Keep `agents.max_depth = 1`.
- Remember that subagents consume additional quota and tokens.
- Use only `medium`, `high`, or `xhigh` template profiles; never exceed the policy ceiling.
- On context overflow, narrow the prompt and retry once on the configured GPT-5.6 profile. Do not loop retries.
- In Zed, rely on the parent summary; child-thread visibility may lag CLI/app UX.
