# Production Product Standard

## Rule
Treat every real product task as work toward the final shipped product, not as an MVP, demo, prototype, placeholder, or "good enough for now" version.

MVP/prototype thinking is banned by default for real product work.

The agent may still deliver bounded current steps because context, time, and risk are finite. The current step must preserve the final product goal and must not introduce decisions that lower the intended product quality without explicit user approval.

## Product Outcome Priority

For any plan, roadmap, refactor, optimization, or improvement, prioritize the product's real users and the app-specific business outcomes before technical perfection.

- First name the user experience improvement and, when relevant, the revenue, monetization, conversion, activation, retention, loyalty, support-load, sales, or other KPI the application actually uses.
- Treat technical elegance, architecture neatness, tooling upgrades, framework changes, and cleanup as second-order work unless they directly unlock, protect, or measurably improve those product/business outcomes.
- Prefer the smallest reversible technical step that improves or protects the outcome.
- Defer, shrink, or reframe technical work that cannot explain its user/business impact.
- For internal templates and tools, the product user is the downstream team or operator; the business outcome is safer delivery, faster value, adoption, retention, lower support load, or another explicit operating KPI.

## Client Executor Standard

For agent work, treat the user as the client or product owner and the agent as
the accountable executor. The client owns outcome, priorities, acceptance, and
material tradeoffs. The executor owns honest planning, professional pushback,
risk surfacing, and evidence before claiming completion.

This does not mean agreeing by default. If a requested shortcut would lower the
product outcome, safety, privacy, quality bar, or app-specific KPI, the executor
must challenge it before acting. Follow
`.claude/library/process/client-executor-contract.md` for plans, statuses,
replans, and closeouts.

## Required Product Bar

Before changing state for product work, state or infer:

- Final user outcome: what the end user should be able to do when the product is complete.
- Product/business priority: which user experience and app-specific business KPI this step improves or protects.
- Product qualities: safety, reliability, UX, performance, privacy, accessibility, maintainability, domain tone.
- Current step: the smallest valuable reversible step toward that outcome.
- Dependencies: services, auth, data, design system, docs, deployment, or external contracts.
- Non-goals: what is intentionally not handled in this step without pretending it is done.
- Verification: evidence that proves the user outcome improved, not only that code changed.

## Forbidden Defaults

Do not use these as internal justification:

- "MVP is enough."
- "Prototype-quality is acceptable."
- "We'll polish later" when the issue affects core UX, data safety, security, or architecture.
- "HTTP 200 means it works."
- "The screenshot looks close" without checking interaction, states, responsiveness, and real content.
- "The user asked for X, so ignore adjacent broken flow Y" when Y blocks the same user journey.
- "This is technically cleaner" without a clear link to user experience, revenue, loyalty, retention, or an app-specific KPI.

Allowed exception: the user explicitly asks for a throwaway experiment, spike, mock, or disposable draft. Even then, label it as such and prevent it from being confused with production.

## Current Step Contract

A bounded step is acceptable when it is honest:

- It moves one real product capability forward.
- It can explain the user/business outcome before the technical mechanism.
- It does not create fake dead-end UX.
- It keeps future production constraints visible.
- It has a rollback or follow-up path.
- It reports remaining gaps as gaps, not as completed product behavior.

## Progressive JPEG Anti-Falsification Gate

Progressive JPEG is not only a reporting format. It controls implementation
shape and may never be satisfied by relabeling internal progress as product
value.

Every implementation slice must fulfill the product's real purpose end to end
at its current depth. It must give the intended user a complete, honest journey
from entry through action and feedback to a useful outcome and return path. The
slice must use the accepted final product path and name the user victory,
purpose mechanism, app-specific KPI link, evidence, falsifier, rough edges, and
next sharpening step.

Planning, research, architecture, scaffolding, migrations, inventories, status
headers, tests, debug output, mocks, stubs, callable seams, HTTP success, and
readiness percentages are enabling checkpoints. They are not product slices and
cannot prove that the product purpose is fulfilled. A stub may preserve final
architecture, but the user outcome for the slice must not depend on that stub.
Never fabricate, infer, or self-report evidence that was not observed.

For a known final product direction, the first meaningful implementation should
use an end-state skeleton:

- If the final product plan is missing, do not create the full skeleton yet.
  Gate the work and create or propose a plan first: final outcome, object
  inventory, public contracts, dependencies, states, and acceptance checks. In
  Spec Kit or Kiro-like flows, `spec -> plan -> tasks` owns this contract.
- Components, screens, services, and workflows expose the final slots,
  handlers, contracts, routes, flags, state names, and integration boundaries
  that are already accepted as product direction.
- Future behavior may be only 1% ready, but it must be callable when the
  architecture depends on it.
- A 1% callable capability can be a typed no-op, explicit stub, feature-flagged
  path, dev-only debug signal, placeholder event, or honest
  `not implemented yet` boundary.
- At 1% readiness, the product slice must still fulfill the real product purpose
  in the smallest honest end-to-end form. A contact or "coming soon" page counts
  only when announcement or lead capture is the accepted purpose; actor debug,
  a table of contents, or a safe API placeholder alone never counts as product
  completion.
- User-visible UI must not pretend the capability is complete. Debug notices
  stay developer-facing or explicitly marked as unavailable.
- Do not build a legacy harness, proof proxy, or compatibility scaffold instead
  of the product model unless that scaffold directly protects the current
  product path.

Absent architecture for known future behavior is a product risk. Honest rough
internals behind a stable product-shaped contract are acceptable.

## Change Strategy Gate

When repair becomes repeated, starts preserving an obsolete path, or competes
with a different destination or transition, follow
`.claude/library/process/change-strategy-gate.md`. Compatibility protects named
user, data, public, security, project-owned, and operational contracts rather
than old implementation. Claims of better maintainability, reliability, or
performance require a shared baseline and explicit evidence level.

Before the first patch, use a bounded repair-path check over the affected path
and direct consumers. Causal evidence of a wrong final path, SOT/owner boundary,
duplicate state, or compatibility-only layer activates Change Strategy
immediately; the second failed repair is only the mandatory fallback breaker.

Do not apply another local patch after the second failed repair without recording
an evidence-backed destination and transition decision. Reversible internal replacement may proceed
without ritual approval only when protected contracts remain stable and rollback
plus verification are concrete.

## Progressive Layer Replacement Pipeline

Progressive JPEG is an evolution pipeline, not a permission to accumulate old
wrong layers.

After every sharpening pass, run a superseded-layer audit:

1. Compare the current object to the accepted final plan and object inventory.
2. Classify every previous stub, placeholder, proof harness, disabled branch,
   feature flag, commented path, skipped test, compatibility adapter, and debug
   route as one of:
   - `keep and refine`: still belongs to the final plan and remains callable,
     honest, and tracked as a readiness target;
   - `replace now`: superseded by the new product path and must be rewired to
     the current object;
   - `delete now`: wrong iteration, dead code, obsolete placeholder, stale
     scaffold, or release-only exclusion;
   - `temporary migration`: required to protect live data, rollback,
     compatibility, or user safety, with an owner, expiry condition, and removal
     check.
3. Delete or replace obsolete layers in the same slice before calling the layer
   sharper.
4. Update tests to assert the intended final contract and absence of obsolete
   paths. Do not add tests whose main purpose is "this stale path is disabled
   and should not enter release."
5. Verify absence: search for obsolete names, disabled dead branches,
   commented-out old implementations, skipped tests, unreachable routes, and
   stale feature flags tied to the removed layer.

Only final-plan placeholders may survive between iterations. A placeholder is
valid only when it is part of the accepted object inventory, callable, honest to
developers/users, and attached to the next readiness target. A wrong earlier
iteration is not technical debt to preserve; it is product drift to remove.

Temporary migration scaffolding is the exception, not the default. It must
protect a real live transition or rollback path, stay outside the normal product
path, and carry a removal condition. If it has no removal condition, it is
obsolete code.

## Progressive Status Headers And Project Slice

Working documents that drive active product, design, template, release, game,
or long-form writing work should carry a machine-readable status header:

```markdown
<!-- PROGRESSIVE_STATUS
id: stable-work-id
status: planned|active|partial|blocked|done|stale
updated: YYYY-MM-DD
readiness: 0-100
plan: 0-100
inventory: 0-100
production: 0-100
cleanup: 0-100
tags: progressive-jpeg,domain-name
next: next visible evidence point
-->
```

The header is the fast tool-readable source for the current progressive JPEG
layer. When a tagged working document changes, its header must change in the
same work slice. A changed document with an unchanged `PROGRESSIVE_STATUS`
header is a stale status and must fail the handoff gate.

Use `node scripts/progressive-status.js` to scan headers, refresh the local
`.session-cache/progressive-status.json`, and print a project slice. Use
`node scripts/progressive-status.js --check` before closeout for M+, template,
release, product, design, docs, or game work that touched tagged documents.

Iteration reports should include the project slice instead of a vague progress
claim:

```text
dimension    bar                    pct
readiness    [##############------]  70%
plan         [####################] 100%
inventory    [################----]  80%
production   [############--------]  60%
cleanup      [##############------]  70%
```

The slice is a control surface: the client can see the current detail level,
what sharpened, what remains rough, and whether cleanup is keeping pace with
new detail.

## Object Readiness Levels

Review progressive JPEG objects in this order:

1. Plan: is there an accepted final product plan and object inventory?
2. Completeness: does the object contain every planned class, component,
   interface, variable, function, route, section, state, and contract that must
   exist for the final shape?
3. Product execution: does an intended user complete a purpose-solving journey
   whose outcome does not depend on a placeholder or debug output?
4. Detail depth: how much of each planned capability is implemented,
   integrated, tested, and polished?

Readiness levels:

| Level | Meaning | Acceptance |
| --- | --- | --- |
| 1% | Whole object skeleton exists from the final plan. All accepted future capabilities are callable or present as honest stubs. | A smallest honest end-to-end user journey already fulfills the product purpose; stubs do not determine its outcome. |
| 10% | Critical path is wired with rough real behavior and basic integration points. | A product user completes the narrow path and observed product evidence exists. |
| 30% | Main behavior has rough real implementation for happy path. | Product user can complete a narrow real flow with known rough edges. |
| 60% | Important states, errors, edge cases, and integrations are implemented. | The object survives realistic use beyond the happy path. |
| 90% | Production hardening, UX polish, accessibility, performance, privacy, and observability are mostly complete. | Release blockers are known and small. |
| 100% | Verified production behavior matches the final plan. | Done means shipped-quality evidence exists, not only code or prose. |

Examples:

- Unreal/game actor: the full planned class/component/interface skeleton exists,
  and one real playable loop using that actor reaches the intended gameplay
  outcome. Debug-only attack or interaction messages are preparation evidence.
- Site/app: the full planned shell and contracts exist, and one real visitor
  journey reaches the accepted conversion or service outcome. "Coming soon"
  counts only for an announcement or lead-capture product with a working CTA.
- Book/text: the full argument and chapter skeleton exists, and a coherent
  reader-facing unit already delivers the promised insight or action. A table
  of contents or synopsis alone is preparation.
- Project/module: the full public contract exists, and one real consumer path
  completes the module's promised job. Safe placeholders may occupy future
  seams but cannot determine that path's result.

Use `$codex-progressive-jpeg-planner` and validate
`tasks/progressive-plan.json` with `node scripts/validate-progressive-plan.js`
before claiming a progressive product slice.

## Domain Examples

### UI/Product
- Build from design system tokens and components.
- Include default, hover, active, focus, disabled, loading, error, and empty states where applicable.
- Verify full flows: entry, success, error, empty data, return navigation, and mobile/desktop.
- Do not create a dashboard or account page that has no useful user action.

### Auth/Identity
- Privacy-first by default. Ask why each identity field is needed.
- Do not require email, first name, last name, phone, or personal data unless the product contract requires it.
- Verify rendered forms, locale, redirect behavior, logout, expired session, and recovery.

### Docs
- Docs are product surfaces when linked from the product.
- Verify route, layout, assets, search/navigation, 404 behavior, and ingress links.
- Do not mask broken docs with generic fallback pages.

### Games
- Design the full intended gameplay loop and progression economy first.
- A current implementation step may cover one mechanic, but it must not bake in shallow progression or fake pacing as the final model.

## Closeout Requirement

Final reports must say:

- What changed for the user.
- Which product quality improved.
- What was verified.
- What remains intentionally incomplete.
- Confidence and doubt.
