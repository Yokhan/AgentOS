---
name: codex-decompose
description: "Break large or risky work into M-sized tasks with dependencies, gates, rollback points, and verification. Trigger on XL tasks, broad architecture changes, or ambiguous 'do everything' requests."
---

# Codex Decompose

Read `.claude/skills/decompose/SKILL.md` for detailed patterns.

## Process

1. Restate the user goal.
2. Identify independent deliverables and dependencies.
3. Split into 3-5 tasks that can each be verified.
4. Define the first safe slice.
   Use `$codex-progressive-jpeg-planner` when the work is a product iteration plan.
5. Define the first useful client-visible view for each slice.
6. Define the acceptance evidence, rough edges, and replan trigger for each slice.
7. Define which accepted future capabilities need 1% callable hooks, contracts, states, events, flags, or no-op stubs in the first slice.
8. Define the object readiness level for each slice: 1%, 10%, 30%, 60%, 90%, or 100%.
9. Define the replacement/cleanup gate for each slice: what previous stubs, wrong iterations, tests, flags, routes, or harnesses must be deleted, replaced, or time-boxed.
   When the slice follows repeated repair or architecture drift, use
   `$codex-change-strategy` to choose destination separately from transition using
   protected contracts and objective evidence.
10. Define which working docs need `PROGRESSIVE_STATUS` and what project-slice bar should change after the slice.
11. State what is deferred and why.

Do not start broad edits before the first slice is clear.
Do not call an internal checkpoint a completed client result unless the user can inspect, accept, or decide from it.
Use progressive JPEG delivery: each slice should sharpen the user's visible picture, not only move hidden internal setup forward.
Use progressive JPEG implementation: each slice should preserve the future product shape when that shape is known, without faking completed behavior.
If the final product plan is missing, create an enabling planning checkpoint and then define the first purpose-solving product slice. Never count the plan itself as product delivery.
If a later slice supersedes an earlier wrong layer, the slice must retire that layer instead of preserving it as disabled legacy.
For tagged working docs, each slice must update the header and pass `node scripts/progressive-status.js --check`.

For client-facing decomposition, follow `.claude/library/process/client-executor-contract.md`.
