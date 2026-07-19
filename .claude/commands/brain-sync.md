You are synchronizing session knowledge to the Obsidian vault (brain/).

## Process

### Step 0: Vault Integrity Check
Before syncing, verify the vault is healthy:
1. Check for empty directories in brain/ (`find brain -type d -empty`)
2. Check brain/00-inbox/ for notes older than 7 days (should be triaged)
3. Check brain/03-knowledge/ for INDEX.md — update it if stale
Report any issues found.

### Step 1: Gather Current Session Context
Collect from this session:
- What tasks were worked on
- What decisions were made
- What problems were encountered and solved
- What patterns or knowledge was discovered
- What's left to do

### Step 2: Update Session Log
Write/update `brain/01-daily/[today's date].md` with:
```markdown
## Session: [time]

### Accomplished
- [What was done]

### Decisions Made
- [Decision]: [Reason]

### Issues Found
- [Issue]: [Resolution or status]

### Next Steps
- [What to do next]
```

### Step 3: Update Project Context
If any project-level information changed, update `brain/02-projects/`:
- Goals, milestones, blockers
- Current feature status

### Step 4: Save Knowledge
If new patterns or lessons were learned, add to `brain/03-knowledge/`:
- Reusable patterns
- Common pitfalls and solutions
- Library/tool findings

### Step 5: Record Decisions
If architectural or significant decisions were made, add to `brain/04-decisions/` using the decision record template.

### Step 6: Confirm
Report what was synced to the user.
