---
name: codex-progressive-jpeg-planner
description: "Plan progressive JPEG iterations for software, games, sites, books, internal tools, modules, and other products so every implementation slice fulfills the product purpose end to end at its declared detail level. Use for iteration plans, readiness layers, whole-product slices, or anti-falsification review."
---

# Codex Progressive JPEG Planner

This skill owns the progressive iteration-planning workflow. Shared product
rules remain the policy SOT; this skill turns them into a plan.

Read:

- `tasks/goal.md` and `tasks/current.md`
- `.claude/library/product/production-product-standard.md`
- `.claude/library/process/product-goal-loop.md`
- `references/domain-examples.md` for the relevant product type

## Hard Rule

Every implementation slice must let the real product user achieve the purpose
for which the product exists through an honest end-to-end path at the slice's
declared detail level.

Architecture, classes, contracts, stubs, debug output, mocks, tests, status
headers, screenshots, HTTP 200, and readiness percentages are enablers or
evidence. None of them is the product outcome by itself. Never relabel an
enabling checkpoint as a product slice.

## Workflow

1. Define product purpose, target user, actual context, current alternative,
   accepted final path, app-specific KPI, and safety/quality guardrails.
2. Inventory the final production object and its accepted contracts. If this is
   unknown, create an `enabling checkpoint`; do not claim a product slice yet.
3. Plan the first whole-product path at low detail. Narrow or use manual
   fulfillment when honest, but preserve entry -> action -> feedback -> useful
   outcome -> return and do not route through a disposable demo.
4. Plan later slices as sharpening passes over the same product path. Each pass
   must add user-visible capability, depth, reliability, trust, reach, or KPI
   performance while still fulfilling the purpose.
5. For every slice specify user victory, journey, purpose mechanism, positioning,
   KPI signal/guardrail, final path, acceptance evidence, strongest falsifier,
   truth boundary, replacement work, rough edges, and next sharpening.
6. Put planning/research/migration/instrumentation work in
   `enabling_checkpoints`; it may protect the outcome but does not count toward
   product-slice delivery.
7. Write the machine-readable contract to `tasks/progressive-plan.json` and run:

```bash
node scripts/validate-progressive-plan.js tasks/progressive-plan.json
```

8. Before accepting a completed slice, inspect real evidence. Expected evidence
   in the plan is not proof that it occurred. Update `PROGRESSIVE_STATUS`, run
   `node scripts/progressive-status.js --check`, and retire superseded paths.
9. When an earlier layer may represent the wrong architecture rather than low
   detail, use `$codex-change-strategy`. Progressive JPEG sharpens the selected
   final path; it does not justify retaining a failed implementation.

## Anti-Falsification Gate

Reject or reclassify the slice when any condition holds:

- The named user cannot complete a meaningful job after the slice.
- The path ends at a stub, debug marker, mock, manual assertion, or dead end.
- Evidence proves only artifact existence, compilation, a test double, HTTP
  success, status text, or self-reported readiness.
- The KPI relationship is decorative rather than behavior -> signal -> outcome.
- The strongest realistic falsifier is omitted or evaded.
- User-visible unavailable behavior is presented as complete.
- A wrong prior layer survives without a migration owner and removal condition.
- The agent would not choose this slice if the internal work had not already
  been done.

Do not fabricate a journey, KPI, evidence reference, user result, or readiness
level to satisfy the schema. Structural validation is a guard, not proof.

## Output

Return:

1. Product purpose and final-path contract.
2. Enabling checkpoints, clearly excluded from product-slice progress.
3. Ordered purpose-solving slices with readiness levels.
4. Acceptance and falsification evidence for each slice.
5. Replacement/cleanup work and replan triggers.
6. The validator result and the remaining product-owner decisions.
