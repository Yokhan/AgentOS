---
name: task-queue
description: "Protocol for external systems to queue tasks for Claude Code agents. Tasks written to tasks/queue.md, picked up at session start or during /sprint."
---

# Task Queue Protocol

External systems (Telegram bots, GitHub Actions, cron jobs, event buses) can queue tasks for Claude Code by writing to `tasks/queue.md`.

## Queue Format

Each queued task is a markdown section:

```markdown
## Queued: 2026-03-24T17:00:00

- **Source**: telegram / github / cron / event-bus
- **Task**: [description of what needs to be done]
- **Issue**: [ID if applicable, e.g., FIT-234]
- **Priority**: P0 (critical) / P1 (high) / P2 (normal) / P3 (low)
- **Agent**: [suggested agent or "auto" for router to decide]
- **Context**: [any additional context, error logs, etc.]
```

## Pickup Protocol

### At Session Start (session-start.sh)
```bash
if [ -f tasks/queue.md ] && [ -s tasks/queue.md ]; then
  QUEUED=$(grep -c "^## Queued:" tasks/queue.md 2>/dev/null) || QUEUED=0
  echo "QUEUE: $QUEUED queued task(s) in tasks/queue.md - review before starting"
fi
```

Agent should:
1. Read `tasks/queue.md`
2. Present queued tasks to user, sorted by priority
3. User approves which tasks to work on
4. Move approved task to `tasks/current.md`, remove from queue.md

### During /sprint
- After completing each task, check `tasks/queue.md` for new entries
- P0 tasks interrupt current work (present to user immediately)
- P1-P3 tasks are picked up in priority order after current task

## Writing to the Queue (for external systems)

### From Telegram Bot
```python
# Append to tasks/queue.md in the project directory
with open(f"{project_path}/tasks/queue.md", "a") as f:
    f.write(f"\n## Queued: {datetime.now().isoformat()}\n")
    f.write(f"- **Source**: telegram\n")
    f.write(f"- **Task**: {task_description}\n")
    f.write(f"- **Issue**: {issue_id}\n")
    f.write(f"- **Priority**: {priority}\n")
    f.write(f"- **Agent**: auto\n\n")
```

### From GitHub Action
```yaml
- name: Queue task for Claude Code
  run: |
    echo "" >> tasks/queue.md
    echo "## Queued: $(date -Iseconds)" >> tasks/queue.md
    echo "- **Source**: github" >> tasks/queue.md
    echo "- **Task**: Deploy failed. Diagnose: ${{ steps.deploy.outputs.error }}" >> tasks/queue.md
    echo "- **Priority**: P0" >> tasks/queue.md
    echo "- **Agent**: devops" >> tasks/queue.md
```

### From Cron/Event Bus
```bash
# Append to queue when project stalls
echo "" >> "$PROJECT_PATH/tasks/queue.md"
echo "## Queued: $(date -Iseconds)" >> "$PROJECT_PATH/tasks/queue.md"
echo "- **Source**: event-bus" >> "$PROJECT_PATH/tasks/queue.md"
echo "- **Task**: Project stalled >10 days. Run scan, identify blockers." >> "$PROJECT_PATH/tasks/queue.md"
echo "- **Priority**: P2" >> "$PROJECT_PATH/tasks/queue.md"
echo "- **Agent**: researcher" >> "$PROJECT_PATH/tasks/queue.md"
```

## Queue Lifecycle
1. External system writes to `tasks/queue.md`
2. Session start notifies user of pending tasks
3. User approves task → moved to `tasks/current.md`
4. Task completed → removed from queue, logged in session
5. Empty queue.md after all tasks processed (or keep as log)

## Priority Guide
- **P0**: Production down, data loss, security breach → interrupt everything
- **P1**: Feature broken for users, deadline approaching → next task
- **P2**: Normal work, improvements, non-urgent bugs → pick up in order
- **P3**: Nice-to-have, exploration, low-impact → when nothing else is queued
