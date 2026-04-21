# Dual-Agent Strategy-Aligned Rollout Plan

## Why This Exists

The dual-agent room was initially planned against an oversimplified pipeline.
AgentOS does not use:

```text
strategy -> plan -> delegation
```

The canonical model is:

```text
Strategy -> Tactic -> Plan -> Todo -> Delegation -> Gate -> Signal -> PA feedback
```

This document defines the corrected rollout so the room integrates with the
real system instead of fighting it.

## Canonical Semantics

### Strategy

- user goal in the real world
- examples: revenue, release target, migration outcome
- not a list of code tasks

### Tactic

- directional approach toward the strategy
- can group projects or categories
- sits between goal and execution planning

### Plan

- concrete plan for one project
- contains ordered or dependent todos

### Todo

- atomic work item
- `agent` todo becomes delegation
- `user` todo stays manual

### Delegation

- actual execution unit for agent work
- feeds gate and downstream signals

## Correct Room Role

The room is not the execution engine.

The room is responsible for:

- framing the problem
- debating the direction
- promoting conclusions into pipeline artifacts
- reviewing execution results
- helping the user arbitrate tradeoffs

The room is not responsible for:

- replacing strategy storage
- replacing plans
- replacing todo state
- executing work outside delegation

## Two Valid User Paths

### Path A: Goal-driven

Use when the user asks for an outcome in the real world.

```text
User prompt
-> room discussion
-> strategy candidate
-> tactic candidates
-> selected tactic
-> per-project plans
-> todos
-> delegations for agent todos
-> gate / signal / feedback
-> room review
```

### Path B: Ad-hoc execution

Use when the user asks for a direct operational task.

```text
User prompt
-> room discussion
-> ad-hoc project plan
-> todos
-> delegations for agent todos
-> gate / signal / feedback
-> room review
```

Rule:

- do not force Path A when Path B is enough

## Rollout

### Phase 0: Align The Model

Scope:

- update all room docs to reference the canonical pipeline
- add session support for linked tactic ids
- keep backward compatibility with current legacy `strategy.plans`

Dependencies:

- none

Acceptance:

- docs stop flattening away tactics
- session state can track tactics even if UI does not yet show them

Risks:

- low code risk
- medium product risk if skipped because every later feature bakes in the wrong
  hierarchy

### Phase 1: Promotion Actions

Scope:

- room can promote a discussion to `Strategy`
- room can promote a discussion to `Ad-hoc Plan`
- room can promote a strategy decision into `Tactic`
- room can promote a tactic decision into project `Plan`

Dependencies:

- Phase 0

Acceptance:

- user can choose the correct downstream artifact instead of only "generate
  strategy"

Risks:

- medium UX risk if promotion choices are unclear
- mitigation: explicit labels and small descriptions

### Phase 2: Artifact Visibility

Scope:

- room shows linked strategies
- room shows linked tactics
- room shows linked plans
- room shows linked delegations
- room feed mirrors only meaningful lifecycle events

Dependencies:

- Phase 1

Acceptance:

- user can see which room discussion produced which pipeline artifacts

Risks:

- event noise
- mitigation: summarize low-level events, keep raw logs secondary

### Phase 3: Todo Discipline

Scope:

- room explicitly chooses `agent` vs `user` for todos
- room captures verify intent before execution
- room uses todo as the execution boundary, not plan or strategy

Dependencies:

- Phase 2

Acceptance:

- delegations are always explainable as agent todos
- user todos never silently become delegations

Risks:

- medium implementation risk because current runtime is still partly legacy
- mitigation: add adapters instead of rewriting all strategy execution at once

### Phase 4: Feedback Projection

Scope:

- inbox projection into room
- signal projection into room
- gate result projection per linked delegation
- strategy/plan/todo status refresh inside room

Dependencies:

- Phase 2
- partial Phase 3

Acceptance:

- room becomes the visible control surface for execution feedback

Risks:

- duplicated truth if room stores derived statuses as canonical
- mitigation: room mirrors pipeline state; pipeline stays canonical

### Phase 5: Write Control

Scope:

- file intents
- single writer policy
- blocked state in room
- lease requests and grants

Dependencies:

- Phase 3 because write permissions must attach to a concrete todo/delegation

Acceptance:

- no write without explicit room-visible intent
- no parallel write in overlapping scope

Risks:

- high UX complexity if introduced before promotion and artifact visibility

### Phase 6: Safe Parallel Work

Scope:

- parallel analysis by default
- parallel writing only for non-overlapping scopes
- optional worktree mode for conflicts

Dependencies:

- Phase 5

Acceptance:

- room can run both agents in parallel without corrupting execution state

Risks:

- highest complexity phase
- should be blocked until earlier phases are stable

## Product Decisions To Keep

### Decision 1: Room-first, pipeline-backed

The room is the visible collaboration surface.
The pipeline remains the system of record.

### Decision 2: Explicit path selection

The room must know whether it is creating:

- a strategy path
- or an ad-hoc plan path

This should be visible in the UI.

### Decision 3: Todo is the execution boundary

Strategy and plan are planning artifacts.
Todo is the handoff point into delegation.

### Decision 4: Tactic must become operational

If tactic stays only in docs and not in room or state, the architecture will
drift again.

## Immediate Code Implications

1. Session state should track linked tactic ids now.
2. Room UI should later show strategy/tactic/plan/delegation links together.
3. Promotion actions should be split into:
   - `promote -> strategy`
   - `promote -> ad-hoc plan`
   - later `promote -> tactic`
   - later `promote -> plan`
4. Delegation projection should be explained as `agent todo execution`, not as a
   direct strategy action.

## Deferred Work

Do not do these before the above phases:

- free-form autonomous debate loops
- parallel writes
- room-owned status truth
- tactic removal for convenience
- provider-specific write behavior without lease discipline

## Exit Criteria

The room is correctly aligned when all of the following are true:

- a user can choose goal-driven or ad-hoc flow
- tactics are visible and first-class
- every delegation can be traced back to an agent todo
- gate and signal results come back into the same room
- write permissions are visible before any parallel execution is allowed
