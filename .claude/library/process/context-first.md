# Context-First Protocol — Know Before You Act

## The Rule
At session start and before any non-trivial task, the agent MUST understand the project it's working in.
Blind action without context = wasted work + broken code.

## Session Start Scan (MANDATORY)

On every session start, before doing ANY work:

1. **Read PROJECT_SPEC.md** — what is this project, what stack, what it provides/depends on
2. **Read tasks/current.md** — what was the last session doing, any handoff context
3. **Read tasks/lessons.md** — avoid repeating past mistakes
4. **Scan recent git log** — `git log --oneline -5` to see what changed recently
5. **Check PROJECT_SPEC.md freshness** — if `last_scan` is older than 7 days, regenerate it

If PROJECT_SPEC.md does NOT exist → generate it immediately (see template in project root).

## PROJECT_SPEC.md Maintenance

### When to update:
- At first session in a new project (auto-generate)
- When stack or dependencies change significantly
- When file structure changes (new modules, renamed dirs)
- When project state changes (MVP → production, monolith → modular)
- At minimum: every 7 days (session-start.sh warns if stale)

### What it contains:
- What this project IS (1-2 sentences)
- Stack and key dependencies
- File structure map (top-level directories with purpose)
- What it provides (APIs, exports, services)
- What it depends on (other projects, external services)
- Current state (active work, phase)
- Last scan date

### Auto-generation protocol:
1. Read package.json / Cargo.toml / go.mod / requirements.txt / pyproject.toml
2. Scan top-level directory structure
3. Read existing CLAUDE.md for project description
4. Check git log for recent activity patterns
5. Write PROJECT_SPEC.md with findings

## Session End Handoff (MANDATORY)

Before ending a session or when work is interrupted, update `tasks/current.md`:

```markdown
## Handoff — [DATE]
### Status: [in-progress / blocked / completed]
### Current file: [path to file being edited]
### What was done: [1-2 sentences]
### What's left: [next steps, ordered]
### Blockers: [what's preventing progress, if any]
### Modified files: [list, or "see git diff"]
### Key decisions: [anything the next session needs to know]
```

This is not optional. Without handoff context, the next session (or a different agent) wastes time re-discovering what was already known.

## Documentation Freshness SLA

| Document | Max staleness | Check method |
|----------|--------------|-------------|
| PROJECT_SPEC.md | 7 days | session-start.sh warns |
| docs/ARCHITECTURE.md | 30 days | check-drift.sh warns |
| docs/API_CONTRACTS.md | On every API change | manual/PR review |
| _reference/tool-registry.md | 14 days | audit-reuse.sh --quick |
| ecosystem.md | 30 days | manual check |

If a document exceeds its SLA → regenerate or update before proceeding with work that depends on it.

## Why This Matters

Without project context, agents:
- Suggest wrong patterns for the stack
- Create files in wrong locations
- Miss existing utilities and duplicate code
- Break conventions established in previous sessions
- Ignore lessons from past mistakes

With project context, agents:
- Work within established conventions from the first action
- Find and reuse existing code
- Make architecture-aware decisions
- Continue previous work seamlessly
