---
name: pipeline
description: "Execute multi-agent pipelines: chain agents in sequence with structured handoffs. Reads pipeline definitions from .claude/pipelines/."
---

# Pipeline Runner

Executes predefined multi-agent workflows. Each pipeline is a sequence of agent steps with gates between them.

## When to Use
- Task naturally involves multiple agent roles (research → implement → review)
- User requests a full lifecycle operation
- Agent router (#1) detects compound signals

## Pipeline Format (.claude/pipelines/*.md)

```markdown
# Pipeline: [name]
## Steps
1. AGENT: [agent_name] | GATE: [condition]
   - Input: [what this step receives]
   - Output: [what this step produces]
2. AGENT: [agent_name] | GATE: [condition]
   ...
```

## Execution Protocol

1. **Load pipeline** — read `.claude/pipelines/{name}.md`
2. **For each step**:
   a. Launch agent (as subagent or in current context)
   b. Provide input from previous step's output
   c. Agent writes output to `tasks/current.md` under `## Pipeline: {name} | Step {N}`
   d. Check gate condition
   e. If gate fails → present to user, wait for guidance
   f. If gate passes → proceed to next step
3. **On completion** — save pipeline metrics to Engram

## Gate Types

| Gate | Condition | On Fail |
|------|-----------|---------|
| `user_approval` | User explicitly approves | Wait for user |
| `typecheck` | `tsc --noEmit` / `mypy` / `cargo check` passes | Fix before proceeding |
| `tests_pass` | All relevant tests pass | Fix before proceeding |
| `verdict:PASS` | Reviewer agent returns PASS | Address feedback, re-review |
| `none` | No gate — proceed automatically | — |

## Handoff Between Steps

Each step writes to `tasks/current.md`:
```markdown
## Pipeline: {name} | Step {N}/{total}
- **Agent**: {agent_name}
- **Status**: DONE
- **Output**: {summary of what was produced}
- **Files**: {list of created/modified files}
- **Confidence**: HIGH/MEDIUM/LOW
- **Next**: Step {N+1} — {next_agent}
```

The next agent reads this block as input context.

## Available Pipelines

See `.claude/pipelines/` for definitions:
- `feature.md` — Full feature lifecycle
- `bugfix.md` — Bug investigation and fix
- `security-patch.md` — Security vulnerability fix

## Creating Project-Specific Pipelines

Projects can create `project-{name}-pipeline.md` in `.claude/pipelines/`.
These are preserved during template sync (project-* prefix).
