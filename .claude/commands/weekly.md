You are running the weekly retrospective and self-improvement cycle.

## Usage
`/weekly`

## Phase 1: Gather Data
1. Read `brain/01-daily/` for the past 7 days
2. Run `git log --oneline --since="7 days ago"`
3. Read `tasks/lessons.md` (count entries)
4. Read `tasks/current.md`

## Phase 2: Analyze + Velocity Metrics
- What was accomplished this week?
- What patterns keep recurring in lessons.md?
- What blocked progress?
- What was discovered/learned?

**Velocity Metrics** (from git history):
```bash
git log --oneline --since="7 days ago" | wc -l        # commits this week
git log --oneline --since="7 days ago"                 # list of commits
grep -r "TODO" src/ 2>/dev/null | wc -l                # open TODOs
grep -c "^###" tasks/lessons.md 2>/dev/null            # lessons count
```
Include in weekly report.

## Phase 3: Self-Improvement Promotion
Run the self-update skill:
1. Find recurring patterns in lessons.md (3+ similar entries)
2. For each pattern:
   - Coding rule → create `.claude/rules/project-[topic].md` (ALWAYS new file with `project-` prefix)
   - Workflow pattern → create `.claude/skills/project-[name]/SKILL.md`
   - Safety check → suggest adding to hooks in `settings.local.json`
3. NEVER modify template files (without `project-` prefix) — they are read-only baseline
4. Archive promoted entries to `brain/03-knowledge/lessons-archive.md`

## Phase 4: Drift Check
Run `bash scripts/check-drift.sh` and address warnings:
- Stale docs? → update or flag
- CLAUDE.md too long? → trim
- lessons.md too long? → promote more aggressively
- Architecture violations? → fix or create task

## Rule Effectiveness Check
For rules promoted in the last month:
1. Count errors in that category BEFORE promotion (from old lessons.md entries)
2. Count errors AFTER promotion
3. If error count didn't decrease: flag rule as "ineffective" for review
Report: Rule name | Errors before | Errors after | Verdict

## Phase 5: Report
Save to `brain/01-daily/weekly-YYYY-MM-DD.md`:
```
# Weekly Retrospective — [date range]

## Accomplished
- [list of completed work]

## Lessons Learned
- [key insights from the week]

## Rules Promoted
- [new rules added to .claude/rules/]

## Skills Updated
- [skills created or modified]

## Drift Status
- [results of check-drift.sh]

## Priorities Next Week
1. [top priority]
2. [second]
3. [third]
```
