---
name: codex-task-queue
description: "Use tasks/queue.md as an agent-agnostic queue for external systems, automation, sprint pickup, and handoffs. Trigger when queueing or picking up tasks."
---

# Codex Task Queue

Use `tasks/queue.md` for queued work from external systems.

## Process

1. Read queued tasks before starting autonomous work.
2. Confirm user approval when required.
3. Move the selected task to `tasks/current.md`.
4. Remove completed or rejected queue entries.
5. Preserve source, priority, issue link, and acceptance criteria.
