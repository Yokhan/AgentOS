# Dual-Agent Child Work Refactor

## Goal

Introduce explicit nested execution entities so the system no longer relies on
implicit links between room sessions and delegations.

Target chain:

```text
Parent Room
-> Project Session
-> Work Item
-> Delegation
-> Gate / Signal / Inbox
-> Parent Room feedback
```

## Why

Before this slice, the runtime had:

- `MultiAgentSession`
- `Delegation`

But it did not have explicit entities for:

- child project execution sessions
- canonical work items produced from room decisions

That made nested orchestration possible in theory but ambiguous in runtime
state.

## New Runtime Entities

### ProjectSession

Represents a child project-scoped execution context created by a parent room.

Fields:

- `parent_room_session_id`
- `project`
- `title`
- `executor_provider`
- `reviewer_provider`
- `linked_work_item_ids`
- `status`

### WorkItem

Represents a canonical unit of work that can later become a delegation.

Fields:

- `parent_room_session_id`
- `project_session_id`
- `project`
- `title`
- `task`
- `executor_provider`
- `reviewer_provider`
- `assignee`
- `status`
- `delegation_id`

## Compatibility Strategy

This slice does not replace delegation execution.

Instead:

- `queue agent task` now creates a `WorkItem` first
- then creates `Delegation`
- then links delegation back to work item and room

This keeps the current execution path working while adding correct origin
tracking.

## UI Implications

The room can now show:

- linked project sessions
- linked work items
- linked delegations

And it can explicitly create:

- child project session
- work item backed agent execution

## Next Step

Use this new foundation to add the real `Todo Composer`:

- `assignee: agent | user`
- `verify` definition
- promotion from room into project session / work item without collapsing
  everything into direct delegation creation
