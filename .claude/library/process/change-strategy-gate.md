# Change Strategy Gate

## Purpose

Stop repeated local patching when the implementation path should change,
without turning "rewrite" into permission to endanger users, data, public
contracts, or release stability.

Compatibility protects verified contracts, not old implementation. The
smallest valuable reversible move is the smallest move toward the accepted
final architecture, not the smallest diff.

This gate is an overlay on the real bugfix, feature, migration, product,
strategy, or template workflow. Its decision record is an enabling checkpoint,
not a product slice.

## Activation

Every bugfix or rework starts with a bounded repair-path check while reading the
affected path and direct consumers. Look for causal evidence of a final-plan,
SOT/owner, state ownership, duplicate implementation, compatibility-layer, or
protected-boundary mismatch. This is diagnosis, not a general architecture
review, decision record, or client checkpoint.

If that reading already proves a qualifying system mismatch, activate the gate
before the first patch. Do not spend one or two repair attempts to confirm an
architectural fact that is already visible in the repository or runtime evidence.

Freeze further implementation changes, except reversible emergency containment,
when any qualifying trigger is present:

- a second repair failed against the same acceptance criterion or a previously
  verified repair recurred;
- the next patch adds a compatibility branch, adapter, fallback, skip,
  stale-path test, or feature flag only to preserve the old path;
- the implementation conflicts with the accepted final plan, SOT, or ownership
  boundary;
- repair has similar blast radius to bounded replacement;
- the agent would not choose the path from the accepted requirements today;
- a breaking data, API, product-behavior, security, release, scope, cost, or
  timeline change is proposed;
- the user asks whether to repair, replace, retire, or migrate.

Skip the full gate for a first isolated leaf defect when this bounded check finds
no causal evidence of a system mismatch or protected-boundary impact. Do not
require a full architecture proof for routine private repairs. After
one failed repair, re-diagnose. After the second, the gate is mandatory even if
the initial scan classified the defect as local.

Invoke this overlay in the existing route as soon as the qualifying evidence is
known. When research changes pipeline, risk, or approval authority, rerun the
semantic task router once with a short discovery record and replace the stale
route state before editing. Do not reroute for wording-only classification
changes or create a nested workflow.

The discovery record uses `phase`, `kind`, `architecture_fit`, `summary`,
`evidence_ref`, `owner`, `sot`, and `protected_boundaries`. A
`repeated-failure` record also requires `acceptance_id`. Run:

```powershell
node scripts/codex-route-task.js "<original request>" --discovery-file <json> --summary --write-state
```

The discovery record is ephemeral router input, not a durable SOT. While its
gate is unresolved, `blockEdits` is true. After the existing orchestrator
artifact contains a valid, unblocked decision whose trigger kind and
`evidence_ref` match that discovery, and `acceptance_id` also matches for a
repeated failure, update the same route state with:

```powershell
node scripts/codex-route-task.js "<original request>" --discovery-file <json> --decision-file <decision.json> --summary --write-state
```

This is a lifecycle resolution of the one discovery reroute, not a second
classification pass. Resume the original bugfix, feature, migration, or other
pipeline; do not leave the task permanently blocked.

Resolve unknowns with bounded read-only discovery first. Ask the client only
when a remaining unknown can materially alter behavior, business KPI, data,
public contracts, ownership, cost, release, security, or irreversible state.

Repeated-failure evidence must identify one acceptance or falsifier ID and two
intervention records: hypothesis, attempted change, before evidence, after
evidence, and `failed` or `recurred` result.

Evaluate this policy by time to verified fix, failed interventions, recurrence,
and avoidable client interruptions. Do not optimize for gate activation count.

## Terrain And Protected Boundaries

Classify the affected system from evidence, not repository age:

| Posture | Default treatment |
| --- | --- |
| `greenfield` | Internal implementation may be disposable only when deployment and consumer evidence shows no live dependency. |
| `evolving` | Preserve named consumers and accepted contracts; replace internals inside those boundaries. |
| `production` | Preserve live behavior, data, public contracts, security, and continuity through a controlled transition. |
| `unknown` | Do not assume compatibility or rewrite authority; unknown material impact is a client checkpoint. |

Inventory each protected boundary with one owner, one active SOT, typed `kind`,
and impact `preserved|changed|removed|unknown`. Valid kinds are
`internal-behavior`, `public-api`, `public-cli`, `public-event`,
`public-config`, `public-file-format`, `user-data`, `security-boundary`, and
`external-dependency`. Do not encode contract type as free text:

- user journey, behavior, accessibility, and product KPI guardrails;
- persisted data, save formats, schemas, and retention obligations;
- public APIs, events, extension points, CLIs, config keys, and file formats;
- security, privacy, compliance, and trust boundaries;
- project-owned overlays and accepted decisions;
- supported environments, deployed integrations, operations, release promises,
  and rollback requirements.

If owner or SOT conflicts, invoke the SOT Conflict Protocol before implementation.
Do not treat tests that encode an internal detail as proof that the detail is a
protected contract.

## Destination And Transition

Do not compare repair, replacement, and migration as peer choices. Declare both:

**Destination**

- `repair`: correct a local defect while keeping the suitable architecture;
- `bounded-replace`: replace an explicitly bounded implementation behind named
  contracts;
- `retire-remove`: remove a capability or path accepted as no longer required.

There is no unbounded `rewrite` destination.

**Transition**

- `direct-swap`: one reversible cutover with protected contracts unchanged;
- `staged-swap`: bounded rollout with checkpoints and rollback;
- `versioned-coexistence`: old and new public contracts coexist through a
  deprecation window;
- `expand-migrate-contract`: expand state, migrate and reconcile, then contract
  only after the removal condition passes.

A production replacement is commonly `bounded-replace + staged-swap`,
`versioned-coexistence`, or `expand-migrate-contract`.

## Non-Compensatory Hard Constraints

Every option records `pass|fail|unknown` plus measured, observed, or
authoritative evidence for:

- product function;
- data safety;
- security and privacy;
- protected contracts;
- verification;
- rollback or recovery.

`fail` eliminates the option. `unknown` blocks the option when its safety
depends on that constraint. Maintainability or performance cannot compensate
for a failed hard constraint. A boolean without an evidence reference is not a
gate result.

## Objective Evidence Matrix

Compare viable options against the same baseline. Mark each dimension
`better|same|worse|unknown|not-applicable` and label support
`measured|observed|authoritative|estimated|unknown`.

Core dimensions:

| Dimension | Useful evidence |
| --- | --- |
| Product outcome | Real user journey, error rate, task success, accessibility, time to value |
| Business outcome | App-specific conversion, retention, revenue, support load, operating cost, delivery lead time |
| Correctness | Reproduction, invariants, contract tests, failure-path coverage |
| Maintainability | Change amplification, coupling, duplicate ownership/SOT/state, diagnostic effort |
| Reliability | Failure modes, races, retries, fallbacks, recovery, incidents |
| Performance | Same workload/data/environment: latency distribution, throughput, CPU, memory, I/O, startup, build, payload |
| Security | Trust boundaries, permissions, input surface, secret handling, recovery impact |
| Transition cost | Implementation, migration, verification, rollout, support, rollback |
| Reversibility | Exercised rollback or recovery appropriate to the affected state |

Optional dimensions include testability, operability, scalability, legacy
removal, and environmental cost when material.

An `estimated` value is a forecast, not an evidence-backed advantage. A
recommendation may cite an advantage only when it is measured, observed, or
authoritative. Lines of code may support analysis but can never be the sole
maintainability metric.

## Conditional Compatibility Profiles

Load only the profiles affected by the option:

- Public API/event/CLI: contract diff, known and unknown consumers, consumer
  tests, versioning, deprecation window, auth/error/rate-limit semantics.
- Persisted data: current/target schema, backup or recovery, representative dry
  run, idempotency/resume, reconciliation, dual-read/write policy when used,
  cutover, and contraction condition.
- External dependency: pinned version, semantic compatibility, failure and
  degradation behavior, and representative integration evidence.

Unknown consumers remain protected; do not infer absence from a search alone.

## Performance Evidence And Total Cost

Do not claim `faster`, `lighter`, or `more scalable` from code shape. A positive
performance claim requires the same relevant workload and dataset, environment
and warm-up policy, baseline and candidate result, threshold or regression
budget, and an executable benchmark, trace, or monitoring reference. Use
p50/p95/p99 when distribution matters. Avoid flaky wall-clock CI thresholds.

Compare categorical `none/S/M/L/unknown` costs with a written basis:

```text
total cost = implementation + transition + verification + rollout/recovery risk
           + expected maintenance + operations + future change amplification
```

Do not fabricate precise percentages or money.

## Decision Authority And Approved Change Envelope

Automatic execution is allowed only inside a current approved envelope that
binds:

- accepted outcome and user behavior;
- permitted ownership boundary and blast radius;
- protected contract IDs;
- destination and allowed transitions;
- risk, downtime, cost, release, and environment limits;
- rollback or recovery requirement;
- approval reference and invalidation conditions.

A normal user implementation request may cover reversible internal choices
inside its accepted scope. Do not ask again for implementation details inside
that envelope. A newer instruction, new consumer, contract change, increased
risk, or exceeded limit invalidates it.

Material behavior, business outcome/KPI, API/data, ownership, release, downtime,
cost, timeline, security, or irreversible-state changes require explicit client approval of the
updated envelope.

## Decision Record And Tool

Store one record in the active orchestrator surface:

- AgentOS Gate when AgentOS owns the task graph;
- existing Spec/Plan/Tasks artifacts when present;
- `tasks/current.md` or optional `tasks/change-strategy.json` in parent-Codex
  projects;
- response-only record for read-only analysis.

When JSON is useful, run:

```bash
node scripts/validate-change-strategy.js tasks/change-strategy.json
```

The validator checks completeness, evidence, envelope scope, and conditional
profiles. It does not choose the architecture. A valid record may remain blocked
pending approval.

## Completion And Cleanup

The chosen workflow may close only after:

- the original acceptance/falsifier passes through the real final path;
- relevant user, contract, data, security, and performance evidence is fresh;
- rollback or recovery is exercised when material;
- the superseded path is removed, or temporary transition scaffolding has an
  owner, removal condition, and absence check;
- monitoring names the fact that reopens this gate.

Characterization may preserve protected behavior while recording known defects
separately. Commits may separate refactoring from behavior change, but the wider
progressive slice cannot close with the obsolete path still active.

## Client Notification

For autonomous reversible internal continuation, notify in one non-blocking
sentence: trigger, chosen destination x transition, and evidence reference. Use
the full decision shape only when approval is required or risk is HIGH/CRITICAL.

```text
Change Strategy Gate
Trigger and acceptance ID:
Project posture and evidence:
Protected boundaries:
Destination x transition options:
Hard constraints:
Evidence: measured | observed | authoritative | estimated | unknown
Recommendation and rejected alternative:
Approved envelope or required decision:
Cleanup, recovery, and regression guard:
```

Notify whenever the gate fires, including when the current envelope permits
automatic continuation.
