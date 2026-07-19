# Product Goal Loop

## Purpose
Keep long product work coherent across turns, compaction, agents, and downstream projects.

This is not a "final slice" model. Agents naturally split work into bounded steps. The goal loop preserves the final product target while each step stays small, reversible, and verifiable.

For client-facing plans, status, replans, and closeouts, also follow
`.claude/library/process/client-executor-contract.md`: the user owns outcome,
priority, and acceptance; the agent owns honest execution, risk surfacing,
professional pushback, and evidence before claiming work is done.

## When Required

Use this loop for:

- M+ tasks.
- Any product, design, auth, data, game, docs, deployment, or template work.
- Any task where the user says "continue", "finish", "make it good", "production", "ecosystem", "goal", "roadmap", or similar.
- Any task after a correction where the prior output missed the user's real goal.

## Language Rule
Plans, audits, checklists, user-facing status, and final reports must use the language of the user's request. Keep code identifiers and commands in their native language.

## Goal Artifact

If `tasks/goal.md` exists, read it before planning. If it does not exist and the work is M+ product work, create or propose it.

The artifact must stay concise and include:

```markdown
# Product Goal

## Final Outcome
[What the finished product lets the end user do.]

## Product/Business Priority
[Primary product user, user experience outcome, and app-specific revenue, loyalty, retention, conversion, activation, or KPI impact when relevant.]

## Quality Bar
[UX, security, privacy, performance, design-system, data, docs, domain constraints.]

## Current Step
[What this turn or task moves forward.]

## Dependencies
[Systems, services, design tokens, APIs, deployment, people, or unknowns.]

## Open Risks
[What could still make the goal fail.]

## Out Of Scope For Current Step
[Honest exclusions, not hidden product debt.]
```

## Operating Loop

1. Restore: read `tasks/goal.md` and `tasks/current.md`.
2. State intent: "User wants", "Success means", "I will verify by".
3. Route: run the project router and load only route-selected skills/rules.
4. Plan: write the current step, product/business outcome link, dependencies, tests, rollback, and progressive JPEG checkpoint into `tasks/current.md`.
5. Execute: implement the step without lowering the final quality bar.
6. Verify: prove behavior through tests, browser/screenshot checks, contract checks, source links, or deployment smoke, depending on domain.
7. Update: mark completed steps, log durable lessons, and update `tasks/goal.md` only when the product goal or quality bar actually changed.

Before choosing a technical improvement, name the product user and the business outcome it improves or protects. Technical perfection, refactoring, tooling, and architecture cleanup are valid only when they directly support user experience, revenue, loyalty, retention, activation, risk reduction, or another app-specific KPI.

Do not claim `Done` unless the verification evidence exists. If evidence is
missing, label the step `Partial`, name the exact gap, and state the next
verification action.

## Progressive JPEG Checkpoint

For M+ work and all status/replan/closeout messages, the current step must give
the client a useful low-resolution view before the final result:

- Current view: what is already inspectable, usable, or decidable.
- Next sharpened layer: which evidence, artifact, or behavior will become clear next.
- Rough edge: what remains incomplete, uncertain, or unverified.
- Replan trigger: which new fact changes scope, deadline, quality bar, or path.

If only internal setup happened, report it as internal setup and name the first
client-visible result. Do not call setup, research, or drafting a delivered
product result unless it creates an inspectable decision point.

For M+, template, release, product, design, docs, game, or long-running work
with tagged working documents, include the progressive project slice from:

```bash
node scripts/progressive-status.js
```

The slice should be shown as a monospace table with aligned ASCII bars for
readiness, plan, inventory, production, and cleanup. Before closeout, run:

```bash
node scripts/progressive-status.js --check
```

If a tagged working document changed but its `PROGRESSIVE_STATUS` header did not
change, the work is not ready to hand off.

## Progressive JPEG Implementation Gate

When the task changes product behavior, architecture, components, screens,
services, or workflows, progressive JPEG also means the implementation keeps the
final product shape visible from the first useful slice.

Before coding, name known future capabilities that belong to the accepted final
outcome. For each one, decide whether it needs a 1% callable contract now:

- If the final product plan is missing, stop implementation and create or
  propose the plan first. The plan must name the final outcome, object
  inventory, public contracts, dependencies, states, and acceptance checks.
- Include it now when later work would otherwise have to replace the component,
  route, data shape, state model, or service boundary.
- Keep it out when the capability is speculative or not part of the accepted
  product direction.
- If included, make it honest: no-op, explicit stub, feature flag, dev-only
  debug signal, placeholder event, or `not implemented yet` boundary.
- Do not expose a fake completed action to the product user.
- Do not spend the slice proving a legacy harness unless it protects the current
  product path or prevents a real regression.

The goal is a low-resolution version of the future product, not a separate demo
path that must be thrown away. Use `$codex-progressive-jpeg-planner` for
iteration planning and validate `tasks/progressive-plan.json` before treating a
step as a product slice.

### Anti-Falsification Gate

Every implementation slice must fulfill the product's real purpose end to end
at its current depth through the accepted final path. It needs a user victory,
entry-to-return journey, purpose mechanism, app-specific KPI link, observed
product evidence, falsifier, truth boundary, rough edges, and next sharpening.

Planning, research, architecture, scaffolding, migration, status, tests, mocks,
stubs, debug output, HTTP success, and inventory completeness are enabling
checkpoints, not product slices. Callable seams preserve architecture but do not
prove user value. The slice outcome must not depend on a stub, and evidence must
never be fabricated or replaced with the agent's own claim.

Verification order for object readiness:

1. Final plan exists.
2. Object inventory matches the plan.
3. A real user completes the smallest honest purpose-solving journey; its
   outcome does not depend on a stub or debug signal.
4. Remaining gaps are classified by detail depth, integration, tests, polish, or
   production hardening.
5. Superseded layers are removed, replaced, or time-boxed as migration
   scaffolding before the next readiness level is claimed.

Progressive layer replacement gate:

- Keep placeholders only when they still belong to the accepted final plan and
  remain callable, honest, and tracked as the next readiness target.
- Replace or delete wrong earlier iterations, obsolete scaffolds, disabled
  branches, stale feature flags, commented-out old implementations, skipped
  tests, and release-only exclusions.
- Do not add tests that merely prove stale code is disabled. Tests should assert
  the intended final contract and, when useful, the absence of obsolete paths.
- Allow temporary migration or rollback scaffolding only when it protects live
  users, data, or compatibility, stays outside the normal product path, and has
  an explicit removal condition.

For example, a game actor needs the planned skeleton plus one real playable loop;
debug-callable methods alone are preparation. A site needs the final shell plus
one real conversion or service journey; "coming soon" counts only when the
accepted product purpose is announcement or lead capture. A book needs the full
argument skeleton plus one coherent unit that already delivers the reader
promise; structure alone is preparation.

## Product Slice Discipline

Current steps must not pretend to be the whole product. Use these labels:

- `Done`: verified and usable as part of the final product.
- `Partial`: intentionally incomplete, with explicit next dependency.
- `Blocked`: cannot progress without user input or external state.
- `Rejected`: would lower the product quality bar or conflict with the goal.

Never call a partial step "done" just because the code compiles.

## Correction Loop

After user correction:

1. Classify the failure: misunderstanding, product gap, design gap, technical bug, process gap, or stale context.
2. Classify the shape: local typo, broken contract, repeated error, architecture/workflow smell, or SOT conflict.
3. For repeated, boundary, architecture, or HIGH-risk failures, name the broken
   link and root-cause hypothesis before editing.
   Record the smallest systemic fix and the regression guard it requires.
4. During reading, run a bounded repair-path check over the affected path and
   direct consumers. If causal system evidence exists, run Change Strategy
   before the first patch. Reroute once only when pipeline, risk, or approval
   authority changes.
5. After a second failed repair, compatibility shim, stale-path test, or
   architecture drift, run `.claude/library/process/change-strategy-gate.md`.
   Compare destination and transition alternatives before another patch.
6. Update `tasks/lessons.md` when the failure is reusable.
7. Re-check the goal and current step before editing again.
8. State what changed in the plan.

Do not keep patching local symptoms when the same error points to a broken
module boundary, stale SOT, missing validator, weak architecture, or failed
feedback loop. Fix the system path or ask for a product-owner decision when the
systemic fix changes scope, ownership, release, timeline, or quality bar.

The Change Strategy Gate may continue automatically for reversible internal
replacement that preserves protected contracts. It must ask the product owner
when user behavior, data, public contracts, security, release, scope, cost,
timeline, or irreversible state changes. "Smallest reversible step" means the
smallest move toward the accepted final system, not the smallest diff.

## Verification Examples

- Product nav: click from main entry to service, authenticate if needed, reach useful screen, return to home, and handle logged-out state.
- Design system: check token tables, component dependencies, rendered geometry, interaction states, and responsive behavior.
- Auth: verify real form fields, locale, redirect, token validation, expired session, logout, and privacy contract.
- Docs: verify linked routes, layout, CSS/JS MIME, 404, and whether docs content answers the user journey.
