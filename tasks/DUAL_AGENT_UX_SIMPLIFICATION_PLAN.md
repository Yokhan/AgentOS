# Dual-Agent UX Simplification Plan

## Goal

Turn the current multi-agent system from an engineering-heavy orchestration console into a usable daily workflow.

The system already has enough runtime capability.

The next work must reduce:

- visible complexity
- user-facing object count
- mode confusion
- accidental exposure of low-level execution controls

This is a corrective product/UX pass, not a capability expansion pass.

## Current Problem

Today the system is internally powerful but externally over-surfaced.

The user sees too much of the control plane:

- room
- work items
- leases
- write access
- child sessions
- batches
- reviews

This breaks the main product promise:

- one context
- one main chat
- optional second agent

Instead, the app currently feels like:

- normal chat
- plus a room
- plus an execution console
- plus an orchestration debugger

That is the wrong product shape.

## Product Model

There should be only 3 visible layers.

### Layer 1: Chat

This is the default.

User expectation:

- talk to orchestrator or project
- get answers
- work normally

Visible objects:

- chat history
- composer
- basic delegation/inbox summary

### Layer 2: Collaborate

This is what `duo` should mean.

User expectation:

- involve the second agent
- compare Claude and Codex
- ask one to challenge the other

Visible objects:

- room feed
- participant presence
- compare/challenge controls
- promotion actions

### Layer 3: Execute

This is only for active implementation work.

User expectation:

- run tasks
- see status
- inspect review outcome

Visible objects:

- work queue
- execution feedback
- review results

Not visible by default:

- raw leases
- child session plumbing
- batch internals
- provider runtime metadata

## Hard Rule

Low-level orchestration objects must stop being first-class UI concepts.

These remain runtime concepts:

- `MultiAgentSession`
- `ProjectSession`
- `WorkItem`
- `FileLease`
- `ParallelBatch`

But user-visible UI should mostly show:

- conversation
- work
- result

This is the main correction.

## Canonical User Flow

### Flow A: Everyday Work

1. User opens orchestrator or project.
2. Chat mode is active.
3. User asks normally.
4. If needed, user toggles `Duo`.
5. User asks both or challenges one side.
6. User returns to chat.

No explicit execution UI is needed here.

### Flow B: Hard Decision

1. User toggles `Duo`.
2. User asks both.
3. Claude frames.
4. Codex critiques.
5. User arbitrates.
6. User promotes result into:
   - strategy
   - or ad-hoc plan

Still no low-level work queue visible unless needed.

### Flow C: Real Execution

1. User enters `Execute`.
2. User sees task/work list.
3. User launches work.
4. System runs execution + review.
5. User sees result.

Only here should execution mechanics become visible.

## UX Corrections

### 1. Reduce Main Navigation

The current visible tabs inside duo should be refactored into:

- `Chat`
- `Collaborate`
- `Execute`

Not:

- `chat`
- `room`
- `work`
- `reviews`

Reason:

- current labels expose implementation structure
- new labels express user intent

### 2. Simplify Composer

The composer currently tries to be a command center.

That is too much.

Target model:

- primary action selector:
  - `Send`
  - `Ask Both`
  - `Challenge`

Secondary actions moved into small contextual menu:

- `@Claude`
- `@Codex`
- `Rebuttal`
- `Promote to Strategy`
- `Promote to Plan`
- `Create Child Session`

Rule:

- keep the composer focused on the 80% path
- move rare actions into secondary UI

### 3. Collapse Execution Surface

In `Execute`, default view should show only:

- task title
- target project
- state
- provider
- reviewer result

Advanced details hidden behind expanders:

- declared paths
- write intent
- lease state
- delegation id
- source links

### 4. Hide Lease Mechanics By Default

Leases are necessary for correctness but too low-level for primary UX.

Default:

- show only conflict summary
- show only “blocked by another write scope”

Advanced:

- actual lease list
- force release
- lease owner

### 5. Hide Child Session Plumbing

Child project sessions should not be a top-level user task.

Default:

- user says “run this in project X”
- system creates/reuses child session implicitly

Advanced:

- explicit child session controls

### 6. Reframe Parallel Work

Parallel work should be phrased as:

- `Run Safe Parallel`

Not:

- `parallel batch`
- `provider lane`
- `batch_id`

Detailed batch structure can remain in debug/details.

## UI Mapping

### Chat Mode

Keep:

- current message list
- normal composer
- inbox/delegation summaries

Hide:

- room feed
- execution panels

### Collaborate Mode

Show:

- compact participant cards
- room feed
- shared working set
- compare/challenge controls
- promotion actions

Hide:

- full work queue by default
- lease internals
- batch internals

### Execute Mode

Show:

- task list
- launch actions
- result cards
- reviewer verdict

Hide by default:

- room chatter
- detailed runtime metadata

## Data/State Simplification

Current state is still too close to internal architecture.

Target UI state should become:

- `chatCollabMode = solo | duo`
- `duoView = chat | collaborate | execute`
- `duoComposerMode = send | ask_both | challenge`
- `duoAdvanced = false`

Avoid exposing state concepts directly tied to runtime internals.

These stay internal:

- `activeDualSession`
- `parallel_batches`
- `active_leases`
- `participant_runtime`

UI should consume them through derived summaries.

## Derived Summaries Needed

Add UI-friendly derived summaries instead of raw structures.

### Collaboration Summary

- who is active
- whether Codex is ready
- latest disagreement
- current working set

### Execution Summary

- ready tasks
- running tasks
- blocked tasks
- review warnings
- review failures

### Conflict Summary

- no conflicts
- write conflict on N files
- blocked by active writer scope

These summaries should replace direct rendering of low-level arrays in the default surface.

## Correct Rollout Order

### Phase 1: Rename and Reframe

Do not change backend.

Only change UX language:

- `room` -> `Collaborate`
- `work/reviews` -> `Execute`
- `ask both / challenge / send`

Acceptance:

- the interface reads like user intent, not runtime topology

### Phase 2: Move Rare Actions Out Of Primary Composer

Keep only:

- `Send`
- `Ask Both`
- `Challenge`

Move the rest into secondary controls.

Acceptance:

- composer becomes understandable without reading docs

### Phase 3: Default-Collapse Low-Level Execution Details

Keep advanced details in expanders:

- leases
- write access
- raw paths
- child session ids
- batch ids

Acceptance:

- execute screen becomes scannable

### Phase 4: Derive User-Facing Summaries

Add summary cards:

- collaboration status
- execution status
- review status
- conflict status

Acceptance:

- user can understand state without opening details

### Phase 5: Make Child Sessions Implicit

Default task creation should infer or reuse child project sessions.

Acceptance:

- child session controls disappear from primary flow

### Phase 6: Advanced Mode

Move the current engineering-heavy controls into:

- `Advanced`
- or `Debug`

Acceptance:

- power remains available
- daily surface becomes much cleaner

## What To Remove From Primary Surface

These should no longer be always visible:

- `grant/revoke write`
- raw lease list
- force release
- parallel batch ids
- provider runtime details
- explicit child session creation

Keep available only in advanced execution details.

## What Must Stay First-Class

These are the correct first-class concepts:

- chat
- second agent
- challenge
- task
- result
- review verdict

Everything else is support machinery.

## Risks

### Risk 1: Hiding Too Much

If too much is hidden too fast, power users lose control.

Mitigation:

- add `Advanced` section instead of deleting controls

### Risk 2: Mixed Semantics During Migration

If labels change but old controls remain equally visible, confusion may increase.

Mitigation:

- rename and collapse in the same pass

### Risk 3: Dual Runtime Still Leaking Through

If raw room/runtime ids still dominate the UI, the product model will remain broken.

Mitigation:

- aggressively replace internal ids with summaries in primary surface

## File-Level Plan

### `src-ui/chat.js`

Main target.

Do:

- simplify duo tabs into user-facing labels
- simplify composer controls
- keep one primary composer
- add secondary action menu

### `src-ui/pages.js`

Do:

- make embedded panels summary-first
- move low-level controls into expandable advanced sections
- keep raw orchestration tools available but collapsed

### `src-ui/store.js`

Do:

- reduce UI state vocabulary to user-facing modes

### `src-ui/api.js`

Do:

- optionally add summary helpers if needed
- avoid leaking internal transport/state semantics into UI labels

## Success Criteria

The simplification pass is successful when:

1. A new user can understand the main workflow without learning internal entity names.
2. `Duo` feels like “bring in a second agent”, not “open orchestration console”.
3. `Execute` feels like “run and inspect work”, not “manually operate the scheduler”.
4. Power features still exist, but are no longer the default surface.

## Bottom Line

The current system should not be expanded further before it is compressed.

The correct next move is:

- less visible machinery
- clearer user language
- fewer first-class UI objects
- stronger separation between
  - talk
  - collaborate
  - execute

That is the right corrective direction.
