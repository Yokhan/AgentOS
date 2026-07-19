---
name: daily-brief
description: "Generate a daily intelligence briefing from session logs, git history, and project state"
---

# Daily Brief Skill

## When to Use
- At the start of a new session
- When the user asks "what happened yesterday" or "catch me up"
- When resuming work after a break

## Process

### 1. Gather Data
Read these sources:
- `brain/01-daily/` — recent session logs (last 3 days)
- `git log --oneline -30` — recent commits
- `brain/02-projects/` — current project goals and blockers
- `brain/03-knowledge/` — recent patterns and lessons

### 2. Generate Brief
Create a brief in `brain/01-daily/[today]-brief.md` using the template from `brain/templates/daily-brief.md`:

```markdown
# Daily Brief: [date]

## Yesterday's Progress
- [What was accomplished]
- [What was discovered]

## Current State
- [Active tasks/features]
- [Known blockers]
- [Test status]

## Today's Priorities
- [Suggested next steps based on context]

## Open Questions
- [Unresolved decisions]
- [Things that need user input]
```

### 3. Technical Metrics
Include in every brief:
- Files > 375 lines: `find src -type f \( -name "*.ts" -o -name "*.py" -o -name "*.go" -o -name "*.rs" \) 2>/dev/null | xargs wc -l 2>/dev/null | awk '$1>250 && $2!="total"' | wc -l` files
- Open TODOs: `grep -r "TODO" src/ 2>/dev/null | wc -l`
- Lessons entries: `grep -c "^###" tasks/lessons.md 2>/dev/null` (flag if >40, promote if >50)

### 4. Present to User
Summarize the brief concisely. Highlight:
- What's blocking
- What needs a decision
- Suggested first task for today
- Any health warnings (large files, growing TODO count)
