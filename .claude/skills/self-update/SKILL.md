---
name: self-update
description: "Self-improvement loop. Trigger when user says improve rules, update skills, promote lessons, or during /weekly retrospective."
---

# Self-Update Skill

Analyze lessons learned and promote recurring patterns into permanent rules and skills.
Based on alirezarezvani/self-improving-agent pattern.

## Three Operations

### /si:review -- Analyze lessons.md
1. Read `tasks/lessons.md`
2. Count total entries
3. Group by category (agent/skill/general)
4. Identify recurring patterns (3+ similar entries)
5. Report findings

### /si:promote -- Graduate patterns to rules
For each recurring pattern (3+ occurrences):
1. Draft a concrete rule
2. ALWAYS create a NEW file with `project-` prefix:
   - `.claude/rules/project-[topic].md` (e.g., `project-no-mock-db.md`, `project-api-retry.md`)
3. NEVER modify template-origin files (files without `project-` prefix) -- they are read-only baseline updated via `/update-template`
4. **Validate new rule**: check that:
   - File is < 375 lines
   - Rule has a concrete do/don't example
   - Rule does not contradict existing rules (grep for conflicting keywords across all rules/)
5. Mark promoted entries in lessons.md with `[PROMOTED]`
6. Move promoted entries to `brain/03-knowledge/lessons-archive.md`

### /si:extract -- Create skills from repeated solutions
For solutions used 3+ times:
1. Identify the reusable workflow
2. Create new SKILL.md in `.claude/skills/project-[name]/SKILL.md` (always `project-` prefix)
3. Include: trigger description, step-by-step process, examples
4. Update CLAUDE.md if new skill changes project workflow

## Constraints
- `tasks/lessons.md` should stay under 200 lines (promote aggressively when over 50 entries)
- Rule files: core guards 28-50 lines, strategic/domain-heavy up to 150 lines
- CLAUDE.md must stay under 300 lines total (1M context limit)
- ALWAYS create new `project-*` files for promotions -- never edit template files
- Every promotion must be logged in brain/03-knowledge/

## Trigger
- During `/weekly` retrospective
- When lessons.md exceeds 50 entries
- On explicit user request ("improve rules", "promote lessons")

---

## Self-Improvement Cycle: Error to Lesson to Pattern to Rule to Habit

This is the maturation pipeline for knowledge. Each stage has different characteristics:

### Stage 1: Error
- A mistake occurs during a session (wrong assumption, failed approach, user correction)
- Captured immediately in `tasks/lessons.md` with Error/Root Cause/Rule format
- Lifespan: single session. Not yet validated as recurring.

### Stage 2: Lesson
- The error entry sits in `tasks/lessons.md` with context
- Other sessions may produce similar entries
- Lifespan: days to weeks. Accumulating evidence.

### Stage 3: Pattern
- 3+ similar lessons identified during `/si:review`
- The recurring theme is named and described: "We keep making X mistake because of Y"
- Lifespan: review cycle. Ready for promotion decision.

### Stage 4: Rule
- Pattern promoted to `.claude/rules/project-[topic].md`
- Written as concrete, testable guidance with do/don't examples
- Loaded automatically every session -- actively prevents the error category
- Lifespan: permanent until explicitly retired or superseded

### Stage 5: Habit
- The rule has been internalized into workflow patterns
- Visible in skills and agent behavior, not just as a rule file
- May evolve into skill modifications, hook additions, or template updates
- Lifespan: indefinite. Part of the system character.

The goal: move errors UP the pipeline as fast as evidence supports. Do not let lessons rot in lessons.md -- promote or archive them.

## When to Promote vs When to Wait

### Promote When (ALL must be true)
1. **3+ occurrences**: the same category of error has appeared at least 3 times (not necessarily identical, but same root cause pattern)
2. **Root cause is understood**: you can explain WHY the error happens, not just WHAT happens
3. **Prevention is actionable**: you can write a concrete rule that would have prevented all 3+ occurrences
4. **Rule is stable**: the proposed rule has not changed between occurrences (if each occurrence suggests a different rule, you do not understand the pattern yet)

### Wait When (ANY of these)
- Only 1-2 occurrences -- could be coincidence, not pattern
- Root cause is unclear -- promoting a misunderstood pattern creates a wrong rule
- The rule would be vague ("be more careful") -- not actionable, not testable
- The environment is still changing (new stack, early project) -- wait for things to settle
- Conflicting evidence: some occurrences suggest rule A, others suggest rule B

### Retire When
- A rule has not been relevant for 5+ sessions (the problem it prevents no longer applies)
- The rule conflicts with a new, better-understood rule
- The project stack/architecture has changed, making the rule obsolete
- Move retired rules to `brain/03-knowledge/lessons-archive.md` with a note on why they were retired

## Rule Quality Criteria

Every promoted rule must pass these checks:

### Specific
- BAD: "Be careful with database queries"
- GOOD: "Always use parameterized queries. Never concatenate user input into SQL strings."

### Testable
- BAD: "Write good error messages"
- GOOD: "Every error response must include: error code (machine-readable), message (human-readable), and request ID (for support correlation)."

### Actionable
- BAD: "Consider performance implications"
- GOOD: "For any query that may return >100 rows: add LIMIT, implement pagination, and add a database index on the WHERE clause columns."

### Includes "Why"
- BAD: "Use Result types instead of exceptions in core modules."
- GOOD: "Use Result types instead of exceptions in core modules -- exceptions create hidden control flow that makes testing and error handling unpredictable. Core must be pure."

### Has a Do/Don't Example
Every rule should include at least one concrete example:
```
DO:   const result = await userService.findById(id); if (result.isErr()) return result;
DONT: try { const user = await userService.findById(id); } catch (e) { /* now what? */ }
```

## Template vs Project Rule Distinction

### Template Rules (no prefix)
- Files in `.claude/rules/` WITHOUT `project-` prefix
- Examples: `architecture.md`, `code-style.md`, `testing.md`, `strategic-thinking.md`
- Origin: the agent-project-template repository
- **Read-only**. NEVER modify these files. They are the baseline updated via `/update-template`.
- They represent universal best practices that apply across all projects.

### Project Rules (`project-` prefix)
- Files in `.claude/rules/` WITH `project-` prefix
- Examples: `project-no-mock-db.md`, `project-api-retry.md`, `project-naming-conventions.md`
- Origin: promoted from this project `tasks/lessons.md`
- **Editable**. These are the project own accumulated wisdom.
- They represent THIS project specific context, patterns, and learned lessons.

### How They Interact
- Claude Code loads router.md from `.claude/rules/` + project-* rules -- both template and project rules are active
- Project rules OVERLAY template rules: if a project rule contradicts a template rule, the project rule takes precedence for this project
- Project rules should reference the template rule they extend or override: "Extends architecture.md: in this project, we also require..."
- When `/update-template` runs, only non-`project-` files are updated. Project rules are preserved.

## Lessons Archive Management

`brain/03-knowledge/lessons-archive.md` is the graveyard and museum of promoted lessons.

### Archive Format
```markdown
### [DATE] -- [Original Title] to [Promoted To]
**Original Error**: [copy from lessons.md]
**Promoted to**: `.claude/rules/project-[name].md` (or skill)
**Promotion date**: [date]
**Status**: ACTIVE | RETIRED ([retirement date], [reason])
```

### Archive Maintenance
- **Monthly**: review archive for retired rules (rules that have not been relevant recently)
- **On conflict**: when two archive entries suggest contradictory rules, investigate which is correct and update the active rule
- **On template update**: after `/update-template`, check if any project rules now duplicate template rules -- retire the project rule if the template covers it

### Archive Size
- No strict limit, but organize by category (architecture, testing, data, API, etc.)
- If a category has 20+ entries, consider whether the rules are too granular and can be consolidated

## Weekly Retrospective Protocol -- Detailed Steps

The `/weekly` command triggers this full protocol:

### Phase 1: Gather (5 min)
1. Read `tasks/lessons.md` -- count entries, identify themes
2. Read recent `brain/01-daily/` session logs -- spot patterns across sessions
3. Read `tasks/current.md` -- check for recurring blockers
4. Count rule files: how many `project-*` rules exist? Are any getting stale?

### Phase 2: Analyze (10 min)
1. Group lessons by category (agent behavior, testing, architecture, tooling, domain)
2. Count occurrences per category -- which category has the most lessons?
3. Identify promotion candidates (3+ similar lessons)
4. Identify retirement candidates (project rules not triggered in 5+ sessions)
5. Check rule quality: do existing project rules meet the Specific/Testable/Actionable/Why criteria?

### Phase 3: Act (10 min)
1. Promote qualifying patterns to `project-*` rules (use `/si:promote`)
2. Extract reusable workflows to skills (use `/si:extract`)
3. Retire stale rules (move to archive with retirement note)
4. Archive promoted lessons (move from lessons.md to lessons-archive.md)
5. Update `tasks/lessons.md` -- should be noticeably shorter after this step

### Phase 4: Report (5 min)
Save retrospective summary to `brain/01-daily/weekly-YYYY-MM-DD.md`:
```markdown
## Weekly Retrospective -- [date]
### Lessons Processed: [X] entries
### Patterns Found: [list with occurrence counts]
### Rules Promoted: [list with file paths]
### Rules Retired: [list with reasons]
### Skills Updated: [list if any]
### Lessons Remaining: [count] (target: <50)
### Key Insight: [one sentence -- the most important meta-learning]
```

## Cross-Reference

- `.claude/library/meta/strategic-thinking.md` -- "Kaizen" (continuous improvement): small consistent improvements compound. "Study, do not just check" (PDCA): understand WHY a lesson emerged, do not just file it.
- `.claude/library/process/self-improvement.md` -- Lessons.md protocol, promotion cycle, anti-patterns to track
- `.claude/library/meta/critical-thinking.md` -- Evidence hierarchy applies to self-improvement too: 3+ occurrences = evidence, 1 occurrence = anecdote
