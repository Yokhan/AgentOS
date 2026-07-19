# Dynamic Task Router

Rules live in `.claude/library/`. They are NOT pre-loaded. Load ONLY what each task needs.

For product, design, auth, data, game, docs, deployment, template, or M+ work, also preserve the v4 product goal loop:

- `.claude/library/product/production-product-standard.md`
- `.claude/library/process/product-goal-loop.md`

Plans, audits, status, and closeout must use the language of the user's request.

## ON EVERY NEW TASK:

1. User gives you a task (any language, any jargon)
2. YOU extract keywords in English: task type + domain + action
   Examples: "ну чёт с логином пиздец" → "fix auth login bug"
            "накидай дизайн экрана" → "design screen layout figma"
            "проверь что тут не так" → "review check code"
3. Call: `get_context(keywords="fix auth login bug")`
   - Default depth=brief → returns MODE + AGENT + file list (~50 tokens)
   - For M+ tasks: `get_context(keywords="...", depth="normal")` → includes full rule text
   - For L/XL tasks: `get_context(keywords="...", depth="full")` → rules + lessons + git + registry + ecosystem
4. Work. Read specific rule files only if you hit a situation that needs them.

## ON TASK CHANGE mid-conversation:

When user switches context:
1. Extract new keywords
2. Call: `switch_context(keywords="new task keywords")`
3. State: "Switching: [old mode] → [new mode], loading [N] rules"

## After compaction:

Call: `get_active_rules()` → restores last routing state.

## If MCP not available (fallback):

`bash scripts/route-task.sh "<keywords>"` → Read listed files manually.

## Manual mode override:

`/mode-code` `/mode-design` `/mode-review` `/mode-research` `/mode-write` `/mode-fix` `/mode-plan`

## MCP Memory (Engram — PROACTIVE):

- Session start: `mem_session_start` + `mem_context`
- After EVERY decision/bug/discovery: `mem_save` immediately
- Before research: `mem_search` first
- On task switch: `mem_save` summary of paused task
- Session end: `mem_session_end`
- No Engram → tasks/lessons.md + brain/ (file fallback)

## Subagent discipline:

Pass task + file paths + findings summary ONLY to subagents.
Subagent calls `get_context()` for its own rules.

## Design work (ALWAYS enforced):

NEVER hardcode visual values. System→Tokens→Components→Screens.
Every container needs layout mode. 8 states for interactive elements.
Search before creating.
For design-system work, also load `.claude/library/domain/domain-design-system.md` and verify token tables, composition traces, Storybook/equivalent stories, and rendered geometry.

## MCP Tools (use these, not bash scripts):

```
get_context(keywords, depth)  — route task + load rules (brief/normal/full)
switch_context(keywords)      — switch task mid-conversation
get_active_rules()            — restore after compaction
research(target)              — auto research: files, importers, git, lessons, registry
verify(size)                  — auto verification: file sizes, syntax, gates
plan_scaffold(task)           — auto plan: affected files, size estimate, template
run_pipeline(name, params?)   — trigger n8n workflow (if n8n running)
list_pipelines()              — list active n8n workflows
pipeline_status(execution_id) — check pipeline execution result
```

## Bash fallbacks (if MCP unavailable):

```
bash scripts/route-task.sh <keywords>
bash scripts/research.sh <path>
bash scripts/verify-check.sh --size M
bash scripts/plan-scaffold.sh <task>
bash scripts/context-restore.sh
```
