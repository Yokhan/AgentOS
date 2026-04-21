# Dual-Agent Pipeline Integration

## Goal

Integrate the live dual-agent room with the existing AgentOS pipeline:

```text
Strategy -> Tactic -> Plan -> Todo -> Delegation -> Gate -> Signal -> PA feedback
```

The room should not become a parallel orchestration universe. It should become
the visible planning and review layer around the pipeline that already exists.

## Core Decision

The correct integration model is:

- `room_session_id` links a room to strategy, plan, and delegation records
- the room must understand the canonical hierarchy:
  `goal/strategy -> tactic -> project plan -> todo -> delegation`
- room receives pipeline lifecycle events
- strategy/tactic/plan/todo/delegation remain canonical execution objects
- the room is the visible collaboration and arbitration layer

This is safer than trying to execute work directly from the room without
touching the existing orchestration pipeline.

## Integration Model

### 1. Strategy

Room role:

- generate goal/strategy candidates
- challenge goal framing and success criteria
- arbitrate strategy before approval

Execution role:

- strategy remains persisted in `.strategies.json`
- strategy remains the user goal layer, not the execution layer

Required linkage:

- `Strategy.room_session_id`

Room events:

- `strategy_linked`

### 2. Tactic

Room role:

- propose tactics under an approved strategy
- challenge whether a tactic is the right path to the goal
- compare tactics across categories or project groups

Execution role:

- tactic remains the direction layer that groups project plans
- tactic is not a chat-only artifact and should not be flattened away in the
  target architecture

Required linkage:

- room should track linked tactic ids even if current runtime is still partially
  legacy and strategy execution still walks `strategy.plans`

Room events:

- `tactic_linked`
- later: `tactic_status_updated`

### 3. Plans

Room role:

- discuss plan shape
- turn a chosen tactic into explicit per-project plans
- review plan progress

Execution role:

- plans remain stored in `tasks/plans/*.json`
- plan remains the project-level container, not the atomic work item

Required linkage:

- `Plan.room_session_id`

Room events:

- `plan_linked`
- `plan_step_updated`

### 4. Todo / Step

Room role:

- challenge todo decomposition
- review assignee choice: `agent` vs `user`
- review verify conditions before execution

Execution role:

- todo/step remains the atomic unit
- only agent-assigned todos become delegations
- user-assigned todos stay manual and should not be forced through delegation

Room events:

- later: `todo_promoted`
- later: `todo_verify_defined`

### 5. Delegations

Room role:

- understand what was delegated
- watch status changes
- review failures and next actions

Execution role:

- delegation remains the actual work unit sent to a project

Required linkage:

- `Delegation.room_session_id`

Room events:

- `delegation_linked`
- `delegation_started`
- `delegation_verifying`
- `delegation_completed`

### 6. Gate

Room role:

- see verify outcome
- compare product and technical interpretation of failure
- decide whether to retry, change plan, or stop

Execution role:

- gate remains canonical verification step

Room events:

- `gate_result`

### 7. Inbox / PA feedback

Room role:

- if a delegation result needs user attention, the room should show it as a
  visible decision point instead of burying it in another pane

Practical note:

- current code already pushes inbox items
- next step should be mirroring relevant inbox items into linked room sessions

### 8. Signals

Room role:

- show relevant warnings and critical signals inside the active room when they
  relate to linked delegations or projects

Practical note:

- current gate->signal path is already working
- next step should be a room-facing projection of critical and warn signals

## Recommended Runtime

### Room-driven planning

```text
User prompt
-> Live room discussion
-> Strategy candidate or ad-hoc plan candidate
-> Strategy approval or ad-hoc plan approval
-> Tactic selection/refinement when strategy path is used
-> Project plan creation
-> Todo decomposition / assignee / verify definition
-> Existing pipeline executes
-> Room watches and reacts
```

### Delegation-driven review

```text
Agent todo queued
-> Delegation created
-> Project executes
-> Gate runs
-> Room receives lifecycle events
-> Claude/Codex can review the outcome
-> User arbitrates if needed
```

## What the room should not own

The room should not become responsible for:

- raw execution scheduling
- replacing delegation storage
- replacing plan storage
- replacing tactic hierarchy with a flat room thread
- forcing every problem through strategy when ad-hoc plan is sufficient
- replacing gate persistence
- directly mutating strategy status outside existing logic

That would duplicate the system and make the product incoherent.

## Canonical User Flows

### Flow A: Goal-driven work

```text
User goal
-> Room discussion
-> Strategy candidate
-> Tactic debate
-> Per-project plans
-> Todos
-> Agent todos become delegations
-> Gate / Signal / PA feedback
-> Room decides next move
```

Use this when the user asks for an outcome in the real world, not a single
project task.

### Flow B: Ad-hoc execution

```text
User project task
-> Room discussion
-> Ad-hoc plan
-> Todos
-> Agent todos become delegations
-> Gate / Signal / PA feedback
-> Room decides next move
```

Use this when the user is asking for a direct operational task and the strategy
layer would only add friction.

## Correct Next Integrations

### Implemented now

- room linkage fields for strategy, tactic, plan, delegation
- room events emitted from strategy queueing and delegation lifecycle

### Next recommended step

1. show linked strategy/tactic/plan/delegation ids in the room UI
2. add explicit room action:
   `promote room conclusion -> generate strategy`
3. add explicit room action:
   `promote room conclusion -> create ad-hoc plan`
4. add explicit room action:
   `promote strategy/tactic conclusion -> create project plan`
5. add explicit room action:
   `promote plan conclusion -> queue todo/delegation`
6. add room projection for inbox items
7. add room projection for warn/critical signals
8. teach room prompts the difference between strategy path and ad-hoc path

### After that

1. room-aware retries for failed delegations
2. room-aware gate review by Claude and Codex
3. room-driven arbitration before retry or escalation
4. tactic-aware execution instead of relying on legacy flat `strategy.plans`

## Dependencies

### Phase 0: Model alignment

- document the canonical pipeline everywhere the room is described
- keep compatibility with current legacy `strategy.plans`
- add session support for linked tactic ids

Why first:

- otherwise every later room feature will encode the wrong hierarchy

### Phase 1: Promotion actions

- room can create strategy
- room can create ad-hoc plan
- room can create project plans from a selected tactic

Depends on:

- phase 0

### Phase 2: Pipeline projection

- room shows linked artifacts and lifecycle
- room mirrors inbox/signals/gate outcomes

Depends on:

- phase 1 for meaningful artifact linkage

### Phase 3: Todo discipline

- room can set assignee and verify intent before execution
- agent todo becomes delegation

Depends on:

- phase 2 so execution feedback is visible

### Phase 4: Write control

- file intents
- single writer
- leases

Depends on:

- phase 3 because write control only matters once room can drive execution

## Risks

### Risk 1: Duplicate truth

If room status and pipeline status are maintained separately, they will drift.

Mitigation:

- pipeline objects stay canonical
- room only links and mirrors

### Risk 2: Overcoupling

If every pipeline step is forced through heavy room interaction, the system
becomes slow.

Mitigation:

- room enriches high-value decisions
- ad-hoc plan path remains first-class for direct tasks

### Risk 3: Event spam

Mirroring every low-level status change into the room can make the feed noisy.

Mitigation:

- only mirror user-meaningful lifecycle events
- keep raw logs in secondary views

### Risk 4: Flattening tactics away

If room integration only links strategy and plan, the tactic layer will keep
existing only in docs and never become operational.

Mitigation:

- track tactic ids in session state now
- add tactic-aware promotion and UI before deeper execution changes

### Risk 5: Wrong default path

If every prompt defaults to strategy generation, simple project work becomes
slower and harder to use.

Mitigation:

- support both `goal-driven strategy path` and `ad-hoc plan path`
- make the room choose explicitly which path it is taking

## Final Recommendation

The live room should become the visible:

- strategy framing layer
- tactic debate layer
- planning layer
- challenge layer
- review layer
- arbitration layer

The existing pipeline should remain the:

- goal/tactic/plan/todo structure
- execution layer
- persistence layer
- verification layer

That separation is the cleanest way to scale the system without rewriting the
whole product.
