---
name: sprint
description: "Autonomous work loop with circuit breaker. Trigger on /sprint command or when user says work autonomously, auto mode, sprint."
---

# Sprint Skill -- Autonomous Work Loop

Combines Ralph Loop (frankbria/ralph-claude-code) + Agentic Sprint (damienlaine/agentic-sprint).
Agent works autonomously through a task list with safety mechanisms.

## Setup
1. Read `tasks/current.md` for task list
2. If no tasks defined, ask user what to work on
3. Prioritize: Critical > High > Medium > Low

## Main Loop

```
FOR each task in priority order:
  1. PLAN   -- evaluate scope, create mini-plan (< 30 changes)
  2. BUILD  -- implement in batches (3-4 files then typecheck)
  3. TEST   -- run relevant tests
  4. VERIFY -- self-critique gate (see VERIFY Phase below)
  5. COMMIT -- conventional commit if tests pass
  6. LOG    -- update tasks/current.md, log to brain/01-daily/
  7. NEXT   -- pick next task
```

## VERIFY Phase (Mandatory between TEST and COMMIT)

After tests pass, before committing — STOP and verify:

1. **Re-read the task description** — the ORIGINAL task, not your plan or interpretation
2. **Diff review** — read your own changes as if reviewing someone else's PR. What would you flag?
3. **Devil's advocate** — name ONE thing that could be wrong with your approach. "Nothing" is not acceptable.
4. **Simpler alternative** — is there a simpler way you dismissed? If yes, why?
5. **Output verification block** in sprint log:
   ```
   VERIFY: [task name]
   - Tests: PASS
   - Matches original intent: YES / PARTIAL / NO
   - Weakest point: [description]
   - Confidence: HIGH / MEDIUM / LOW
   ```
6. **Gate**: If "Matches original intent" is NO or Confidence is LOW → do NOT commit. Re-enter PLAN phase for this task.

This phase exists because the agent CAN find its own errors (proven by finding them immediately when asked). The problem is not capability but activation. This phase activates self-critique.

## Exit Conditions (BOTH required -- dual-condition gate)
1. `completion_indicators >= 2`:
   - Tests pass for the task
   - Task marked done in current.md
2. AND `EXIT_SIGNAL = true`:
   - All tasks in current.md are done
   - OR user sends stop signal
   - OR circuit breaker triggered

## Circuit Breaker (CRITICAL SAFETY)

### Adaptive Limits (scale with task size)

|                | XS  | S   | M    | L    | XL   |
|----------------|-----|-----|------|------|------|
| tool_calls     | 20  | 50  | 100  | 200  | 300  |
| wall_clock     | 5m  | 15m | 60m  | 120m | 180m |
| no_progress    | 2   | 2   | 3    | 4    | 5    |
| same_error     | 2   | 3   | 5    | 5    | 7    |
| file_edits     | 3   | 5   | 10   | 15   | 20   |

Determine task size at sprint start (see implementer.md Task Size Classification).

### Escalation Chain (instead of immediate STOP)

When a circuit breaker condition triggers, do NOT stop immediately. Escalate:

**Level 1: Self-Diagnose (2 min max)**
- Read last 3 error messages
- Identify pattern: same error? different errors? no progress?
- If diagnosis reveals a simple fix → apply and continue

**Level 2: Escalate to Another Agent**
- Same error 3x → invoke **simplifier** (decompose the problem into smaller parts)
- No progress → invoke **researcher** (find alternative approach, search memory globally)
- If the other agent provides a solution → return to implementer with new context

**Level 3: STOP with Post-Mortem**
- If Level 2 also fails → STOP
- Generate mandatory post-mortem (see below)
- Present to user with clear next step

### Post-Mortem (MANDATORY on any STOP)

Write to `tasks/current.md`:
```markdown
## Circuit Breaker Post-Mortem — [date]
- **Trigger**: [what condition caused STOP]
- **Task**: [what was being worked on]
- **Timeline**: [what was tried, in order]
- **Root cause**: [best guess at why it failed]
- **What would help**: [specific next step for user or next session]
- **Prevention**: [candidate rule for lessons.md]
```

Also save to Engram: `mem_save(topic_key="post-mortem:{task_slug}", ...)`
Also write lesson to `tasks/lessons.md` with Error → Root Cause → Rule format.

## Sprint Resume
If sprint is interrupted, save state to `tasks/sprint-state.md`:
- Current task and progress
- Completed tasks list
- Pending tasks list
- Circuit breaker counters

At next sprint start: if `tasks/sprint-state.md` exists and is < 24 hours old, offer to resume.

## On Any Stop
1. Save full state to `tasks/current.md`:
   - What was in progress
   - What was completed
   - Where it stopped and why
   - Next steps for human/next session
2. Log sprint summary to `brain/01-daily/sprint-YYYY-MM-DD.md`
3. Add any errors to `tasks/lessons.md`

## Logging
- Every task start: log to brain/01-daily/
- Every completion: update tasks/current.md
- Every error: tasks/lessons.md
- Sprint summary: brain/01-daily/sprint-YYYY-MM-DD.md

## Constraints
- Max task scope: 30 related changes (Working Memory Cliff)
- Larger tasks: decompose into subtasks before starting
- If uncertain about a decision: log to `tasks/current.md` as blocker, skip to next task
- NEVER force-push, delete data, or make irreversible changes in autonomous mode

---

## OODA Integration -- Each Sprint Iteration Is One OODA Cycle

Every task in the main loop maps to an OODA cycle:

| OODA Phase | Sprint Phase | What Happens |
|------------|-------------|-------------|
| **Observe** | PLAN | Read task description, scan affected files, check test state, review related lessons.md entries |
| **Orient** | PLAN | Determine WHY this task matters (Commander Intent). Identify center of gravity -- the one change that unlocks the rest. Assess: is the scope realistic? |
| **Decide** | PLAN to BUILD | Choose approach. If multiple approaches exist, pick the one with fastest feedback loop. Write mini-plan: ordered steps, expected outcomes, abort criteria. |
| **Act** | BUILD to TEST to COMMIT | Execute the plan in small batches. After each batch: validate (typecheck, test). Feed results back into Observe for the next micro-cycle. |

The full sprint is a series of nested OODA loops:
- **Strategic loop** (whole sprint): Am I working on the right tasks in the right order?
- **Operational loop** (per task): Is my approach to this task working?
- **Tactical loop** (per file/function): Is this specific change correct?

If any loop produces a surprise, escalate one level up and re-orient before continuing.

## Commander Intent Check -- Sprint Start Protocol

Before touching any code, answer these questions explicitly (log answers to sprint state):

1. **What is the user trying to achieve overall?** Not "implement feature X" but "enable users to Y so that Z improves." Two levels up from the task.
2. **What does DONE look like?** Concrete, observable criteria. "Tests pass" is necessary but not sufficient -- what behavior should the user see?
3. **What would make this sprint a failure even if all tasks complete?** (e.g., breaking existing functionality, creating tech debt that blocks next sprint, missing the actual user need)
4. **What is explicitly OUT of scope?** Name it so you do not drift into it.
5. **What decisions can I make autonomously vs. what needs user input?** Architectural changes, API contract changes, data model changes = ask. Implementation details within established patterns = proceed.

If you cannot answer #1, STOP and ask the user before starting the sprint.

## Energy Management -- Task Ordering Within a Sprint

Not all tasks are equal in cognitive demand. Front-load hard tasks when context is fresh:

### Sprint Task Order
1. **Complex/creative tasks first** -- New architecture, tricky algorithms, design decisions. These need maximum cognitive resources and fresh context.
2. **Integration tasks second** -- Connecting components, wiring up APIs, resolving merge conflicts. These need broad context awareness.
3. **Mechanical tasks third** -- Adding tests for existing code, fixing lint warnings, updating documentation. These need focus but not creativity.
4. **Polish tasks last** -- Formatting, naming improvements, comment updates, cleanup. These are low-risk and do not suffer from fatigue.

### Context Preservation
- After completing a complex task, capture key decisions and patterns discovered in the sprint log BEFORE moving to the next task. Context evaporates fast.
- If a complex task requires a break (circuit breaker, need human input), save a detailed "where I was" note: current hypothesis, what has been tried, next step to try.

## Progress Tracking -- Concrete Metrics Per Iteration

After each task completion, log these metrics to the sprint state:

```
### Task: [name]
- Files changed: [count] ([list])
- Tests: [passed]/[total] ([new tests added])
- Coverage delta: [before]% to [after]% (for affected modules)
- Duration: [minutes]
- Circuit breaker events: [count and type]
- Approach changes: [0 = smooth, 1+ = note what pivoted]
```

Cumulative sprint metrics (updated after each task):
- **Velocity**: tasks completed / tasks planned
- **Health**: circuit breaker triggers (0 = healthy, 1-2 = watch, 3+ = systemic issue)
- **Waste**: approach changes + reverted commits (lower is better)
- **Coverage trend**: is overall coverage going up or down?

These metrics are not targets -- they are DIAGNOSTICS. If velocity is low, the question is "why?" not "work faster."

## Pivot Triggers -- When to Change Approach Mid-Sprint

### The 3-Failure Rule
If the same approach fails 3 times in a row:
1. STOP executing
2. Re-enter ORIENT phase explicitly
3. List what you have tried and why each failed
4. Generate at least 2 alternative approaches
5. Pick the approach with the shortest feedback loop
6. If no alternatives exist, log as blocker and move to next task

### Other Pivot Signals
- **Scope explosion**: what seemed like a 5-file change now touches 15+ files -- decompose or defer
- **Dependency discovery**: the task requires changes to a module you did not expect -- re-assess priority and impact
- **Test cascade**: fixing one test breaks two others -- the design is wrong, not the test. Step back to architecture.
- **Time overrun**: task taking 3x longer than estimated -- either the estimate was wrong (update it) or the approach is wrong (pivot)
- **Diminishing returns**: each iteration improves things less -- you have hit the culmination point (Clausewitz). Ship what you have.

### Pivot Protocol
```
1. Log current state (what is done, what is broken, what is pending)
2. Commit any working code (even partial -- use WIP commit)
3. Document WHY the pivot happened in sprint log
4. Re-plan from current state, not from original plan
5. Resume main loop with updated approach
```

## Sprint Retrospective -- Mini-Protocol (End of Every Sprint)

Before closing the sprint, run this 5-minute retrospective:

### What Worked
- Which approaches were effective? (Extract as patterns for future sprints)
- Which tasks went smoother than expected? (Why? Can that be replicated?)
- Were there any reusable solutions worth extracting to skills?

### What Did Not Work
- Where did the circuit breaker trigger? (Root cause, not just symptom)
- Which estimates were most wrong? (Calibration data for future sprints)
- Were there unnecessary back-and-forth cycles? (Process improvement opportunity)

### Carry Forward
- Unfinished tasks: update `tasks/current.md` with current state and next steps
- Lessons learned: add to `tasks/lessons.md` (Error then Root Cause then Rule format)
- Pattern candidates: if a lesson appeared 3+ times across recent sprints, flag for promotion to rule
- Tech debt created: log explicitly so it does not become invisible

### Retrospective Output
Save to `brain/01-daily/sprint-YYYY-MM-DD.md`:
```
## Sprint Retrospective -- [date]
### Completed: [list with metrics]
### Blocked/Deferred: [list with reasons]
### Key Lesson: [one sentence -- the most important thing learned]
### Process Change: [one concrete change for next sprint, if any]
```

## Cross-Agent Coordination

When the sprint involves work that benefits from specialized agents:

| Situation | Agent to Invoke | When |
|-----------|----------------|------|
| Code review needed before merge | `reviewer` | After BUILD, before final COMMIT |
| Test coverage is below threshold | `test-engineer` | After BUILD, during TEST phase |
| API contract changes detected | `api-contract` skill | When modifying endpoint signatures |
| Database schema changes | `migrate` skill | When adding/modifying data models |
| Security-sensitive changes | `reviewer` with security focus | Auth, crypto, input validation changes |
| Performance-critical paths | Profiler + `reviewer` | Hot paths, data processing, query optimization |

Coordination protocol:
1. Complete your BUILD phase to a stable, committable state
2. Invoke the specialized agent/skill with clear context: what changed, why, what to focus on
3. Incorporate feedback before continuing to next task
4. Log coordination events in sprint state for retrospective

## Cross-Reference

- `.claude/library/meta/strategic-thinking.md` -- OODA loop mechanics, Commander Intent, center of gravity, culmination point, flow over batch (small batches with validation), know when to stop
- `.claude/library/conflict/conflict-resolution.md` -- Circuit breaker rules for file edit limits and test failure limits
- `.claude/library/process/self-improvement.md` -- Lessons logging protocol, promotion cycle
