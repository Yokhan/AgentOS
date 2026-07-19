# Codex Fan-Out Patterns

Date: 2026-07-11

## Purpose

Use this document when a Codex task can be split across `.codex/agents` workers.

The goal is not to force a single spec system into every project. The goal is to
reuse the best Spec Kit ideas in a flexible way:

- Work from durable artifacts when they exist: `spec.md`, `plan.md`, `tasks.md`,
  `PROJECT_SPEC.md`, `tasks/current.md`, issue text, or domain-specific equivalents.
- Let existing project systems win: Spec Kit, litkit, Kiro, AgentOS, or project-local
  `project-*` skills and agents.
- Use explicit parallelism markers or non-overlapping file ownership before spawning
  write-capable work.
- Keep the parent Codex thread responsible for sequencing, consolidation, and final
  verification.

The user does not need to request subagents explicitly. After routing, the
parent evaluates the returned `fanout` contract and automatically starts useful
independent lanes when they improve wall-clock time, evidence, or context
isolation. The parent reports who was spawned and why. Explicit user opt-out
always wins.

## Agent Policy Source Of Truth

`scripts/codex-agent-policy.js` owns the template's role, model, reasoning,
sandbox, fan-out limit, and parent-default boundaries. Files under
`.codex/agents/*.toml` are runtime declarations validated against that policy.

| Role | Model | Effort | Default use |
| --- | --- | --- | --- |
| `scout` | GPT-5.6 Luna | `low` | Bounded file/symbol discovery without synthesis |
| `log_analyst` | GPT-5.6 Luna | `low` | Bounded failure/log extraction and grouping |
| `summarizer` | GPT-5.6 Luna | `low` | Condense completed evidence without new judgment |
| `pr_explorer` | GPT-5.6 Terra | `medium` | Repository map and dependency trace |
| `docs_researcher` | GPT-5.6 Terra | `medium` | Fresh official documentation |
| `tester` | GPT-5.6 Terra | `medium` | Test and regression strategy |
| `implementer` | GPT-5.6 Terra | `high` | Exact isolated write scope only |
| `reviewer` | GPT-5.6 Sol | `high` | Correctness and regression review |
| `design_reviewer` | GPT-5.6 Sol | `high` | UI, UX, design-system, accessibility |
| `product_reviewer` | GPT-5.6 Sol | `high` | User outcome, KPI, offer, and journey |
| `security_reviewer` | GPT-5.6 Sol | `xhigh` | Security and trust boundaries |
| `systems_reviewer` | GPT-5.6 Sol | `xhigh` | SOT, architecture, and repeated failures |

The parent model stays in user or IDE configuration. Template profiles never
exceed `xhigh`.

## Automatic Fan-Out Decision

The router returns `required`, `recommended`, `conditional`, or `skip`.

- `required`: high-risk state-changing work needs independent verification.
- `recommended`: the task contains explicit independent work with material
  parallel value; spawn only the useful lanes while the parent continues.
- `conditional`: one specialist is available; spawn only when it is a
  non-blocking sidecar with material value. Candidate count alone produces at
  most `conditional`.
- `skip`: direct XS question, no specialist, or explicit user opt-out.

Candidates are not a command to duplicate work. Before each spawn confirm that
the lane has a narrow output, does not block the parent's immediate next step,
does not repeat another lane, and costs less wall-clock attention than serial
execution. Read-only work is the default. Write delegation requires exact,
non-overlapping files or `[P]` ownership.

Automatic fan-out is limited to one wave. Children do not recursively fan out,
and the parent must not start a second automatic wave to compensate for weak
prompts or premature delegation.

### Runtime Evidence Gate

A parent marker or prose claim does not prove that a custom subagent ran. For a
runtime check, `node scripts/validate-subagent-trace.js` must observe one
correlated chain: parent thread, genuine spawn event, distinct child thread ID,
required role/model metadata, child activity or completion, and a wait that
references the same child. If the runtime cannot expose this evidence, report
the profile as unverified and do not claim role/model isolation.

Before spawning any write-capable batch, pass the proposed `{ agent, files }`
assignments through `validateWriteAssignments` from
`scripts/codex-agent-policy.js`. Missing scopes, duplicate files, and
directory/file containment conflicts block write fan-out.

## Discovery First

Before spawning subagents, inspect the project for existing workflow artifacts:

For Codex, start with the deterministic route check:

```bash
node scripts/codex-route-task.js "<user request>" --summary --write-state
```

Use the returned skills, subagents, pipeline, risk, and orchestrator owner as the parent-thread contract. The script does not replace project artifacts; it makes the first routing step explicit and testable.

| Look for | Meaning |
| --- | --- |
| `.specify/`, `specs/*/spec.md`, `specs/*/plan.md`, `specs/*/tasks.md` | Spec Kit or Spec Kit-like flow |
| `litkit/`, `core/config.yaml`, domain `project-*` skills | Domain pipeline owns phases and vocabulary |
| `.kiro/`, `requirements.md`, `design.md`, `tasks.md` | Kiro-style artifact flow |
| `PROJECT_SPEC.md`, `tasks/current.md`, `docs/AGENT_PIPELINES.md` | Template-native context |
| `AgentOS` metadata, Strategy/Tactic/Plan/Todo/Gate terms | AgentOS owns orchestration state |

If a project has its own task graph, do not replace it. Treat it as the input
contract for Codex fan-out.

## Spec-Kit-Inspired Flow

Spec Kit's portable pattern is:

1. Spec: define the user-facing behavior and acceptance criteria.
2. Plan: map architecture, constraints, risks, and verification.
3. Tasks: create dependency-ordered tasks and mark independent work with `[P]`.
4. Implement: execute in dependency order, using parallel work only where safe.

Codex adaptation:

- Use `pr_explorer` and `docs_researcher` to ground the spec and plan in the repo.
- Use `tester`, `reviewer`, `security_reviewer`, and `design_reviewer` as gates.
- Use `[P]` tasks or explicit non-overlapping files as the only default write split.
- Use `implementer` only for isolated tasks with exact files and expected behavior.

## Routing Matrix

| Work type | Default agents | Parent action |
| --- | --- | --- |
| Bugfix | `scout`, `log_analyst`, `tester`, `reviewer` | Reproduce, patch, run regression check |
| Security patch | `security_reviewer`, `tester` | Patch narrowly, prove exploit path is closed |
| UI/design | `design_reviewer`, `tester`, `reviewer` | Apply token/component/state fixes, screenshot-check |
| API/framework docs | `docs_researcher`, `reviewer` | Browse official docs when freshness matters, update code/docs |
| Large feature | `pr_explorer`, `docs_researcher`, `tester`, then optional `reviewer` | Build spec/plan/tasks before edits |
| Product/GTM | `product_reviewer`, `scout`, `reviewer` | Tie recommendations to user value, KPI, proof, and journey |
| Architecture/SOT | `systems_reviewer`, `scout`, `reviewer` | Resolve ownership and broken contracts before local edits |
| Existing `tasks.md` | `scout`, `tester` | Identify dependency order and `[P]` groups |
| Release or migration | `reviewer`, `tester`, optional `security_reviewer` | Validate compatibility and rollback path |

## Prompt Templates

### Feature Fan-Out

```text
Use Codex subagents with a Spec/Plan/Tasks flow.

First inspect whether this project already has Spec Kit, litkit, Kiro, AgentOS,
or project-local workflow artifacts. Do not replace them.

Spawn:
- pr_explorer: map affected files, current behavior, dependencies, and tests.
- docs_researcher: verify official docs or source-backed APIs if external behavior matters.
- tester: propose acceptance and regression checks.

Continue independent parent work. Wait only when the next action needs a child
result. Parent thread writes the plan and performs edits unless there are
explicit [P] tasks with non-overlapping files.
```

### Task Decomposition

```text
Read the existing spec/plan/tasks artifacts.
Spawn pr_explorer and tester to identify:
- dependency order
- tasks that are truly parallel-safe
- tasks that must not be split
- missing acceptance or regression checks

Return a revised task graph using [P] only for independent work.
Do not edit files during this pass.
```

### Review Fan-Out

```text
Review this branch/change. Spawn:
- pr_explorer for changed paths and execution flow.
- reviewer for correctness, regressions, boundaries, and missing tests.
- tester for the smallest useful verification suite.
- security_reviewer only if auth, secrets, input handling, permissions, storage,
  or dependency risk is touched.

Continue independent parent work, then consolidate completed findings by severity.
```

### Security Fan-Out

```text
Spawn:
- security_reviewer: identify concrete exploit paths and trust-boundary failures.
- pr_explorer: map affected files and callers.
- tester: define checks that prove vulnerable and allowed behavior.

Parent thread applies the smallest patch and verifies the exploit path is closed.
```

### Design Fan-Out

```text
Spawn:
- design_reviewer: audit UX, UI, tokens, components, states, responsive behavior,
  accessibility, and visual consistency.
- tester: define viewport, state, overflow, and interaction checks.
- reviewer: check implementation boundaries and regressions.

Parent thread edits after results are merged.
```

### Debug Fan-Out

```text
Spawn:
- pr_explorer: trace failing path, callers, config, and recent history.
- tester: define the minimal reproduction and regression check.
- reviewer: check likely fix risks before editing.

Do not patch until the failure is observable or the blocker is explicitly documented.
```

## Write Delegation Rules

Default policy: exploration and review roles request read-only sandboxes, and the
parent edits. This is a workflow rule, not a security boundary: Codex reapplies
the parent's live sandbox and approval settings to children. Treat role prompts
and TOML defaults as defense in depth, and keep the parent responsible for
reviewing every child result before it changes the product.

Use `implementer` only when all are true:

- The task maps to exact files.
- No other worker is editing those files.
- Inputs and expected behavior are stated in one prompt.
- The parent can run or inspect a narrow verification after the worker returns.

Do not use `implementer` for shared architecture, broad refactors, migrations with
hidden coupling, security-sensitive patches, or UI work that needs screenshot-driven
iteration across many components.

## AgentOS Compatibility

AgentOS has its own orchestration vocabulary and runtime loops. Keep Codex fan-out
additive:

- Do not require AgentOS schema changes for this template.
- Do not rename setup or sync entrypoints.
- Treat AgentOS Strategy/Tactic/Plan/Todo/Gate artifacts as the source task graph
  when present.
- If `codex-route-task.js` reports `orchestrator.owner = agentos`, Codex acts as
  a worker for the AgentOS task graph and does not create a competing graph.
- Ship new Codex docs, skills, and agents through the existing template setup/sync
  allowlist only.
- AgentOS may choose a template release tag, but projects still consume that tag
  through `scripts/sync-template.sh --from-git --ref <tag>`.

## Zed Note

Zed can run Codex through ACP and Codex can load native `.codex/agents` project
configuration in a trusted repo. Zed may not expose child-thread details as clearly
as the Codex CLI/app. Rely on the parent summary during normal work and use
`scripts/test-codex-subagents-live.sh --yes` only when runtime verification is worth
spending quota.

## Sources

- OpenAI GPT-5.6 release: https://openai.com/index/gpt-5-6/
- OpenAI Codex subagents: https://developers.openai.com/codex/subagents
- OpenAI multi-agent guide: https://developers.openai.com/api/docs/guides/tools-multi-agent
- GitHub Spec Kit: https://github.com/github/spec-kit
- Spec Kit documentation: https://github.github.io/spec-kit/
- Spec-driven overview: https://github.com/github/spec-kit/blob/main/spec-driven.md
