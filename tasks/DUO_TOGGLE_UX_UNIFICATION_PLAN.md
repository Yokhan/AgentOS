# Duo Toggle UX Unification Plan

## Problem

Current multi-agent runtime is operational, but the UX model is wrong for daily use:

- the main shell is still `dashboard + project detail + chat sidebar`
- dual-agent collaboration is exposed as a separate `duo` screen through `showDualAgents`
- the user mental model is simpler: one orchestrator, one project context, one chat, optional second agent

This creates needless mode-switching:

- user must decide between "normal chat" and "duo screen"
- room state feels detached from the canonical chat context
- the app looks like two products glued together: `chat` and `duo`

The next major UX refactor should unify them into one chat shell with a simple collaboration toggle.

## Canonical Product Model

There is only one active context at a time:

- `_orchestrator`
- or one concrete project

Inside that context, the user works in one chat shell.

That shell can run in one of these collaboration modes:

- `solo`
- `duo`

Optional later overlay:

- `auto_review`

Important boundary:

- `solo/duo` is a chat-layer choice
- `strategy/tactic/plan/todo/delegation/gate/signal` remain the execution system of record
- `duo` does not replace the pipeline

## Current State In Code

Relevant current behavior:

- `src-ui/views.js`
  - `showDualAgents` switches the whole main panel to `DualAgentsView`
  - header button `duo` behaves like a route
- `src-ui/store.js`
  - `showDualAgents`, `activeDualSession`, `dualSessionData`, `dualHistories`, `dualBusy`
- `src-ui/chat.js`
  - `ChatSidebar` is still the canonical chat shell for orchestrator and projects
  - this is the natural place for a `solo/duo` toggle
- `src-ui/pages.js`
  - `DualAgentsView` already contains the collaboration features:
    - ask both
    - mentions
    - challenge/rebuttal
    - work items
    - write access
    - leases
    - parallel rounds
    - auto reviews
- backend is already sufficient for a unified UX:
  - sessions
  - room events
  - work items
  - file scope
  - leases
  - safe parallel batches
  - reviewer loop

Conclusion:

- backend is ahead of frontend
- the main refactor now is UX composition, not orchestration capability

## Target UX

### One Shell

The user always sees:

- main content on the left
- chat shell on the right

Main content stays as-is:

- dashboard
- project detail
- strategy
- plans
- graph
- settings

The chat shell remains the primary control surface.

### Chat Header

The chat header gets a collaboration switch:

- `Solo`
- `Duo`

When `Solo`:

- current chat behaves exactly like today
- standard message list is primary
- no second-agent room panel is shown

When `Duo`:

- the same context upgrades into collaboration mode
- a hidden dual session is created or reused for that context
- the chat shell shows:
  - primary chat stream
  - duo sidecar / tabs / collapsible room feed
  - second-agent controls

The user should never have to think:

- "Should I go to another screen?"

The only question should be:

- "Do I want a second agent involved right now?"

### Minimal Daily UX

For daily operation, the user flow should be:

1. Open app.
2. Choose orchestrator or project.
3. Use the normal chat.
4. If needed, turn on `Duo`.
5. Ask both, challenge, queue work, review results.
6. Turn `Duo` off when done.

No separate navigation step should be required.

## User Stories

### Story 1: Normal Orchestrator Work

The user opens `_orchestrator`.

- `Solo` is default
- chat behaves exactly like legacy orchestrator chat

When a task becomes strategic or ambiguous:

- user flips to `Duo`
- Claude and Codex both become visible in the same shell
- room can promote discussion into strategy or ad-hoc plan

### Story 2: Project Debugging

The user opens one project.

- `Solo` remains default
- user chats as before

If the bug is complex:

- flip to `Duo`
- run `ask both`
- compare product framing and technical critique
- create work items from the same shell

### Story 3: Parallel Execution

The user is still in one project or one orchestrator-linked flow.

- `Duo` is on
- both providers can be write-enabled
- work items with non-overlapping scopes are queued from the same shell
- room feed shows leases, conflicts, batch progress, reviewer output

The user should not need to open a special "multi-agent screen" to do this.

### Story 4: Strategy Path

The user discusses a goal in the orchestrator chat.

- Duo is on
- Claude frames the goal
- Codex challenges feasibility
- user arbitrates
- room output is promoted into:
  - `Strategy`
  - then `Tactic`
  - then `Plan`
  - then `Todo/WorkItem`

The conversation and the pipeline stay visibly connected.

## Architectural Direction

### Principle 1: Chat Is Canonical, Room Is Embedded

The canonical top-level UI remains `ChatSidebar`.

The live room becomes an embedded collaboration layer.

This means:

- no separate `DualAgentsView` as the primary UX
- reuse current dual runtime
- relocate its controls into the chat shell incrementally

### Principle 2: Keep One Input, Not Two Products

There should be one input box for the current context.

In `Solo` mode:

- input targets the classic chat pipeline

In `Duo` mode:

- the same input gets targeting controls:
  - `ask claude`
  - `ask codex`
  - `ask both`
  - `challenge`
  - `rebuttal`

This should feel like one upgraded composer, not another page.

### Principle 3: Preserve The Existing Main Layout

Do not move orchestration controls into the left content pane.

Reason:

- dashboard and project detail already use the left pane
- the right pane is already the interaction hub
- changing both sides at once would increase blast radius

### Principle 4: Room Feed Is A Sidecar, Not The Whole App

In `Duo` mode, the chat shell should gain a secondary collaboration surface:

- preferred v1: tabbed or split sidecar inside the right pane

Suggested structure:

- top: chat header + `Solo/Duo`
- middle tabs:
  - `Chat`
  - `Room`
  - `Work`
  - `Reviews`
- bottom: one shared composer

This is materially simpler than replacing the entire screen with `DualAgentsView`.

## State Model Refactor

### Current

Current UI state is route-like:

- `showDualAgents`
- `activeDualSession`
- `dualSessionData`

### Target

Replace route semantics with collaboration semantics.

Suggested state:

- `chatCollabMode = "solo" | "duo"`
- `dualSessionByContext`
- `activeRoomTab = "chat" | "room" | "work" | "reviews"`
- `duoExpanded = true | false`

Context key:

- `_orchestrator`
- or project name

Meaning:

- the dual session is a hidden runtime object attached to context
- not a navigation destination

### Migration Note

`showDualAgents` can survive temporarily as an internal compatibility flag during migration.

But the goal should be:

- header no longer navigates to `DualAgentsView`
- header only toggles collaboration mode

## Screen Composition Plan

### Phase A: Overlay, Not Rewrite

Fastest safe path:

- keep `ChatSidebar` as root
- add `Solo/Duo` switch in `ChatSidebar` header
- when duo is enabled:
  - show extra room tabs below the standard message stream
  - or use a split within the sidebar

This avoids ripping apart `views.js` first.

### Phase B: Extract Shared Duo Panels

Move reusable panels out of `DualAgentsView`:

- room feed
- participant presence
- write access panel
- work item queue
- auto review panel
- parallel rounds panel

Then reuse them inside the chat shell.

### Phase C: Remove Standalone Duo Screen

Only after the embedded version is stable:

- deprecate `DualAgentsView`
- remove `showDualAgents` as a primary route
- convert header `duo` button into a plain mode toggle

## Pipeline Integration In The Unified UX

The unified shell should make the pipeline clearer, not blur it.

### In Solo

User mainly sees:

- normal chat
- delegations
- inbox
- running banner

### In Duo

Additional embedded sections appear:

- `Room`
  - interleaved public agent messages
  - mentions
  - challenge/rebuttal
- `Work`
  - work items
  - scopes
  - write access
  - leases
  - batches
- `Pipeline`
  - linked strategies
  - linked tactics
  - linked plans
  - linked delegations
- `Reviews`
  - auto reviews
  - verdicts
  - signals/inbox derived from reviews

This preserves the canonical pipeline:

- discussion in room
- promotion into strategy/plan/todo
- execution via work item/delegation
- feedback via gate/signal/review

## Child Project Sessions

The nested model should remain, but it should become less visible as a separate concept.

User expectation:

- "I asked the orchestrator to involve Codex in project X"

System reality:

- parent room
- child project session
- work items
- delegations

UX rule:

- expose child project sessions only when operationally needed
- do not force the user to think in these objects during normal chatting

Recommended UI:

- show child project sessions as linked artifacts in the `Work` tab
- not as a first-class navigation mode

## Risks

### Risk 1: Two Sources Of Truth

If the app keeps:

- classic chat history
- separate room history
- separate duo screen

the user will not know which stream matters.

Mitigation:

- keep classic chat primary
- keep room feed secondary but attached to same context
- avoid multiple top-level routes for the same collaboration state

### Risk 2: Sidebar Overload

The chat sidebar is already dense:

- inbox
- delegations
- attachments
- streaming
- model/effort selectors

Adding all duo controls directly will create clutter.

Mitigation:

- use tabs or collapsible sections
- default to compact `Duo` mode
- only expand advanced controls on demand

### Risk 3: Premature Transcript Merge

Trying to fully merge:

- normal chat transcript
- room feed
- system events

in one single message list will create confusion fast.

Mitigation:

- keep them visually separate at first
- one shell, multiple tabs

### Risk 4: Polling And Performance

The current app already polls dual session data.

If embedded poorly:

- constant duo polling in all contexts could waste cycles

Mitigation:

- only poll room state when:
  - `chatCollabMode === "duo"`
  - and current context has an active dual session

### Risk 5: Strategy Confusion

If `duo` is presented as "the planning system", the user will misunderstand the pipeline.

Mitigation:

- clearly label promotion actions:
  - `promote to strategy`
  - `promote to ad-hoc plan`
- keep linked pipeline artifacts visible

## Dependencies

### UX Refactor Dependencies

Needed first:

- stable embedded room panels
- context-keyed session lookup
- mode toggle in chat shell

Nice-to-have later:

- better reviewer verdict UX
- auto policy on review fail/warn

### Backend Dependencies

No major new backend architecture is required for the toggle refactor.

That is good news.

Needed adjustments are mostly UI-facing:

- more context-keyed session convenience helpers
- possible lighter payloads for compact embedded mode

## Recommended Rollout

### Phase 0: Spec Alignment

Done by this document.

### Phase 1: Embedded Duo Toggle

Goal:

- `Solo/Duo` switch in `ChatSidebar`
- same context
- no route switch

Changes:

- add `chatCollabMode`
- keep `showDualAgents` only as temporary compatibility
- auto-create/reuse dual session per context when toggled on
- keep classic chat visible

Acceptance:

- user can stay in orchestrator/project chat and enable duo without leaving the screen

### Phase 2: Room Tabs In Chat Shell

Goal:

- expose embedded collaboration panels

Tabs:

- `Chat`
- `Room`
- `Work`
- `Reviews`

Acceptance:

- user can ask both, see room activity, manage work items, and inspect reviews from the same shell

### Phase 3: Promote Actions From Embedded Shell

Goal:

- allow strategy/plan/work creation from the same chat shell

Acceptance:

- no need to open a separate duo page to start orchestration

### Phase 4: Retire Standalone Duo Screen

Goal:

- remove `showDualAgents` as primary navigation

Acceptance:

- header `duo` no longer swaps the whole main panel
- collaboration is truly a mode, not a page

### Phase 5: Polish

Goal:

- compress complexity for daily use

Additions:

- compact mode
- contextual hints
- default `Solo`
- remember last mode per context if useful

## Exact File-Level Refactor Plan

### `src-ui/store.js`

Add:

- `chatCollabMode`
- `activeRoomTab`
- `duoExpanded`
- optional `dualSessionByContext`

Deprecate later:

- `showDualAgents`

### `src-ui/chat.js`

Primary target for the UX refactor.

Add:

- `Solo/Duo` switch in `ChatSidebar` header
- embedded room tab strip
- integrated composer targeting controls
- compact collaboration summary in header

Extract or mount:

- room feed panel
- work panel
- reviews panel

### `src-ui/pages.js`

Refactor `DualAgentsView` into reusable subcomponents:

- `DuoPresencePanel`
- `DuoRoomFeed`
- `DuoWorkPanel`
- `DuoReviewPanel`
- `DuoPipelineLinks`

Then:

- keep `DualAgentsView` only as legacy wrapper during transition

### `src-ui/views.js`

Change header `duo` button behavior:

- from route toggle
- to collaboration mode toggle

Later:

- remove `showDualAgents` from `App()` main view switching

### `src-ui/api.js`

Add convenience helpers:

- `toggleChatDuoMode(context)`
- `getOrCreateContextDualSession(context)`

Keep existing session/work/review APIs.

### `src-ui/app.js`

Change auto-open logic:

- stop auto-routing to duo
- if desired, only prewarm or prepare the dual session for orchestrator
- do not switch visible layout automatically

## Recommendation

Do not attempt a "perfect merge" of all message systems first.

Best practical move:

1. keep the current chat sidebar canonical
2. add a `Solo/Duo` toggle there
3. embed duo panels under tabs
4. reuse existing backend and room runtime
5. only then retire the standalone duo page

This is the highest-value, lowest-risk path.

## Bottom Line

The system should feel like:

- one app
- one chat
- one context
- optional second agent

Not:

- one app
- one chat
- plus another hidden app called `duo`

That is the correct direction for the next major refactor.
