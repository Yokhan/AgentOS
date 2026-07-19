---
name: codex-pipeline-workflow
description: "Execute shared agent pipelines in Codex: feature, bugfix, security patch, design, review, and closeout workflows with explicit gates. Use when a task needs multiple phases, risk control, verification, or the user asks to run a pipeline."
---

# Codex Pipeline Workflow

Use `docs/AGENT_PIPELINES.md` as the source of truth.

## Process

1. Run `node scripts/codex-route-task.js "<user request>" --summary --write-state` unless the user already supplied a route.
2. Identify the pipeline: feature, bugfix, security patch, design, review, release, or template maintenance.
3. Read the relevant pipeline section from `docs/AGENT_PIPELINES.md`.
4. State the active phase and gate before doing work.
5. Complete one phase at a time.
6. Stop at user-approval gates when required by risk or local rules.
7. Write handoff notes to `tasks/current.md` for M+ or interrupted work.
8. Run the verification commands from the pipeline, then close out with confidence and doubt.

## Codex Adaptation

Do not reuse Claude model routing. Follow the Codex route's fan-out decision and `scripts/codex-agent-policy.js`. Spawn independent Codex specialists proactively when they reduce wall-clock time, isolate noisy context, or add a required risk gate; the parent still owns sequencing, edits, and final verification.
