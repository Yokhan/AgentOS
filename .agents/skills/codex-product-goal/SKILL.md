---
name: codex-product-goal
description: "Maintain a goal-like product contract for Codex tasks: final outcome, quality bar, current step, dependencies, risks, language matching, and verification. Use for M+ work, continue/finish requests, product strategy, or corrections after missed intent."
---

# Codex Product Goal

Read:

- `.claude/library/product/production-product-standard.md`
- `.claude/library/process/product-goal-loop.md`
- `.claude/library/process/client-executor-contract.md`
- `tasks/goal.md` when present
- `tasks/current.md`

## Process

1. State the user's real outcome in the user's language.
2. Name the product user and product/business outcome before the technical approach.
3. Preserve the final product goal and quality bar.
4. Define the current bounded step without pretending it completes the whole product.
5. Treat the user as the client/product owner and the agent as the accountable executor.
6. List dependencies, risks, acceptance evidence, and honest out-of-scope items.
7. Use progressive JPEG delivery: first useful view, next sharpened evidence layer, rough edges, and replan trigger.
   For iteration planning, delegate the detailed slice contract to `$codex-progressive-jpeg-planner`.
8. If the final product plan is missing, gate implementation and create/propose the plan first.
9. For known final capabilities, require an end-state skeleton: 1% callable hooks, slots, contracts, feature flags, no-op stubs, or dev-only debug signals when the architecture depends on them.
10. After each sharpening pass, run a superseded-layer audit and delete, replace, or time-box obsolete layers.
    If repair has repeated or the architecture no longer matches the accepted
    final product, use `$codex-change-strategy` before changing the path.
11. When working docs carry `PROGRESSIVE_STATUS`, report the project slice with `node scripts/progressive-status.js` and run `node scripts/progressive-status.js --check` before closeout.
12. Update `tasks/current.md` before edits for M+ work.
13. Update `tasks/goal.md` only when the final outcome, product/business priority, or quality bar changes.
14. Verify the user outcome, not just file changes.

## Gates

- Do not use MVP/prototype reasoning unless explicitly requested.
- Technical improvements must directly unlock, protect, or measurably improve user experience, revenue, loyalty, retention, conversion, activation, support load, or another app-specific KPI.
- Do not lower UX, security, privacy, data, or architecture quality to make the step easier.
- Do not agree by default when a request conflicts with evidence, product outcome, safety, quality, or app-specific KPI.
- Do not claim work is done, tested, reviewed, researched, or released without fresh evidence or a cited existing artifact.
- Do not use legacy harness proof as a substitute for the product model unless it protects the current product path.
- Do not judge detail depth before checking that object inventory matches the final product plan.
- Do not preserve wrong earlier iterations as disabled legacy, stale placeholders, commented-out code, skipped tests, or release-only exclusion harnesses.
- Do not fake user-visible readiness for a 1% callable stub; mark rough edges honestly.
- Do not call planning, architecture, debug output, status, or stub inventory a product slice. Every implementation slice must fulfill the product purpose through an end-to-end user path.
- Do not close out changed tagged working documents when their `PROGRESSIVE_STATUS` header is stale.
- Do not preserve implementation merely for compatibility. Preserve verified
  user, data, public, security, project-owned, and operational contracts; use
  objective evidence before claiming replacement is better.
- Plans, audits, and final reports use the language of the user's request.
- Partial work must be reported as partial with the next dependency.
- Status and closeout messages must show what is sharp now, what is still rough, what evidence comes next, and what fact would force a replan.

## Object Readiness Check

For game actors, sites, books, docs, modules, and other product objects:

1. Plan exists: final production function, inventory, contracts, dependencies, states, and acceptance checks.
2. Object is complete in shape: planned classes/components/interfaces/sections/functions/routes exist.
3. Object is executable: at 1% it performs the smallest honest production function.
4. Depth is labeled: stub/debug, rough happy path, integrated, hardened, or production-ready.
5. Superseded layers are handled: keep only final-plan placeholders; replace/delete wrong layers; time-box migration scaffolding with a removal condition.
