You are running an autonomous work loop with circuit breaker safety.

## Usage
`/sprint` — work through tasks/current.md
`/sprint [task]` — work on specific task

## Based on
- Ralph Loop (frankbria/ralph-claude-code) — dual-condition exit
- Agentic Sprint (damienlaine/agentic-sprint) — spec-driven iteration

## Setup
1. Read `tasks/current.md` for task list
2. Read `tasks/lessons.md` to avoid known mistakes
3. If no tasks → ask user what to work on
4. Pick highest priority unfinished task

## Main Loop
```
FOR each task:
  1. PLAN   — mini-plan, max 30 changes (Working Memory Cliff)
  2. BUILD  — implement in batches (3-4 files → typecheck)
  3. TEST   — run relevant tests
  4. COMMIT — conventional commit if green
  5. LOG    — update tasks/current.md
  6. NEXT   — pick next task
```

## Exit Conditions (BOTH required)
1. completion_indicators >= 2 (tests pass + task marked done)
2. AND EXIT_SIGNAL = true (all tasks done / user stop / circuit breaker)

## Circuit Breaker
- 3 no-progress loops → STOP
- 5 same-error loops → STOP
- 100 tool calls this sprint → STOP
- Same test failing > 3 times → STOP
- Same file edited > 5 times → WARNING: possible loop
- Same file edited > 10 times → STOP

## On Any Stop
1. Save state to `tasks/current.md` (what was in progress, where stopped, why, next steps)
2. Log sprint summary to `brain/01-daily/sprint-YYYY-MM-DD.md`
3. Errors → `tasks/lessons.md`

## SAFETY: NEVER in autonomous mode
- Force-push to any branch
- Delete data or databases
- Make irreversible changes
- Skip tests before commit
