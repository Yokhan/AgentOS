# Self-Improvement Rules

## lessons.md Protocol

After EVERY user correction or discovered mistake:

1. Open `tasks/lessons.md`
2. Add entry in this format:
   ```
   ### [DATE] — [Brief Title]
   **Track**: BUG | KNOWLEDGE | PATTERN | PROCESS
   **Severity**: P0 | P1 | P2 | P3
   **Error**: What went wrong
   **Root cause**: Why it happened
   **Rule**: Concrete prevention rule
   **Applies to**: [agent/skill/general]
   **Category**: [security | architecture | testing | workflow | tooling | performance | ux | general]
   **Status**: ACTIVE
   ```
3. Track definitions: BUG (code broke), KNOWLEDGE (didn't know), PATTERN (recurring anti-pattern), PROCESS (workflow failed).
4. Severity: P0 (data loss/security), P1 (user-facing bug), P2 (minor/suboptimal), P3 (cosmetic).
5. Never skip this step. This is how the system learns.

## Correction Classifier (BEFORE logging)

When user corrects you, classify the correction type first:

| Type | Signal | Action |
|------|--------|--------|
| **STYLE** | "Use camelCase" / "shorter names" / "no comments" | Save to brain/02-projects/ as user preference. Do NOT log to lessons.md. |
| **BUG** | "This breaks X" / "regression" / "data loss" | Log to lessons.md + Engram. Immediate fix + prevention rule. |
| **KNOWLEDGE_GAP** | "That API works differently" / "wrong syntax" | Log to lessons.md + Engram. Update domain knowledge. |
| **DESIGN_DISAGREEMENT** | "I think we should use X instead" | Present trade-offs. User decides. Save to brain/04-decisions/. |
| **MISUNDERSTANDING** | "No, I meant the OTHER file" | Clarify and retry. No logging needed. |

Only BUG and KNOWLEDGE_GAP go to lessons.md. This keeps signal-to-noise ratio high.

**Mapping Classifier → Track field** (when logging to lessons.md):
- BUG → Track: **BUG**
- KNOWLEDGE_GAP → Track: **KNOWLEDGE**
- Recurring BUG/KNOWLEDGE_GAP (3+ similar entries) → Track: **PATTERN**
- If the failure was in the workflow itself (skipped step, wrong ceremony) → Track: **PROCESS**

## Dual-Write Protocol

After writing to lessons.md, ALSO save to Engram (if available):
```
mem_save(topic_key="lesson:{brief_slug}", category="learning",
  content="Error: ... Root cause: ... Rule: ...")
```
This makes lessons searchable via `mem_search` across sessions and projects.

## At Session Start

1. Read `tasks/lessons.md` — avoid repeating past mistakes
2. Read `tasks/current.md` — continue where last session left off
3. Check `brain/01-daily/` for recent session logs if context needed
4. Initialize MCP memory if available:
   - Call `mcp__engram__mem_session_start` with project name and date
   - If Engram unavailable: memory operations degrade to file-only (tasks/lessons.md, brain/). See memory-router fallback.

## Promotion Cycle

When `tasks/lessons.md` exceeds 50 entries:
- Identify recurring patterns (3+ similar errors)
- Promote to `.claude/rules/project-[topic].md` as permanent rules (ALWAYS create new `project-` prefixed files)
- Extract reusable solutions into `.claude/skills/project-[name]/SKILL.md`
- NEVER modify template rules (files without `project-` prefix) — they are the read-only baseline updated via `/update-template`
- Archive promoted entries (move to `brain/03-knowledge/lessons-archive.md`)
- Use `/weekly` command to trigger this process
- Project rules overlay and complement template rules. Claude Code loads ALL files from `rules/`.

## Anti-Patterns to Track

- Hallucinated APIs or packages → add to lessons with source verification
- Files growing > 375 lines → note which module and why
- Tests skipped before commit → note the context
- Deep imports bypassing index → note the offending path

## Feedback Loop Speed
- The faster the cycle Error -> Lesson -> Rule, the faster the entire system improves.
- Ideal: mistake happens -> lesson recorded in same session -> rule applied from next action onward.
- Never batch lessons for "later" — context decays rapidly. Write it while the pain is fresh.
- Slow feedback loops cause the same mistake to repeat 3-5 times before correction. That's waste.
- Measure: how many sessions between a mistake and the rule that prevents it? Target: zero.

## Meta-Lessons: HOW the System Fails
Track not just WHAT went wrong, but the PATTERN of failure:
- **Knowledge gap**: didn't know the API/tool/pattern existed. Fix: add to domain knowledge.
- **Attention failure**: knew the rule but didn't check. Fix: add to pre-commit hook or checklist.
- **Wrong mental model**: understood the system incorrectly. Fix: update architecture docs, add test.
- **Communication gap**: user meant X, agent understood Y. Fix: improve clarification protocol.
- **Overconfidence**: was certain about something wrong. Fix: add verification step for that category.
- Categorizing failure modes reveals which SYSTEM improvement has highest leverage.

### Sycophancy Failure
When the user points out a flaw the agent should have caught:
- Log: what the flaw was, what verification step SHOULD have caught it
- Fix: strengthen the self-verification gate for that category
- Track frequency: if >3 sycophancy failures in same category → verification protocol for that category is insufficient, needs redesign
- This is the HIGHEST PRIORITY failure type — it means the system is blind to its own errors

### Commitment Bias Failure
When the agent defended a wrong approach for 2+ iterations before correcting:
- Log: what the wrong approach was, what the correct one was, when the pivot should have happened
- Fix: add the specific scenario to the Sunk Cost Test examples
- Root cause: almost always "completion bias" — the agent wanted to be DONE more than it wanted to be RIGHT
- Prevention: the Mid-Build Checkpoint and Pre-Output Verification Gate in implementer.md

## Cross-Project vs Project-Specific Learning
- **Universal lessons** (apply everywhere): naming conventions, error handling patterns, git workflow, testing strategy.
  Promote to template rules (`.claude/rules/` without `project-` prefix, via `/update-template`).
- **Project-specific lessons** (apply here only): this API's quirks, this DB's schema, this team's conventions.
  Keep in `project-*.md` rules or `tasks/lessons.md`.
- Before promoting: ask "would this help on a DIFFERENT project?" If yes -> universal. If no -> project-specific.
- Universal rules are higher leverage — one lesson improves ALL future projects.

## Quality Over Quantity
- 5 precise, actionable rules beat 20 vague guidelines.
- Each rule must be: specific (clear when it applies), actionable (clear what to do), testable (clear when violated).
- Bad rule: "Write good error messages." Good rule: "Every error includes machine code + human message + correlation ID."
- If a rule can't be violated, it's not a rule — it's a platitude. Delete it.
- Periodically audit rules: can I give a concrete example of this rule preventing a mistake? If no -> sharpen or delete.

## Retirement Protocol
- Rules that no longer apply must be ARCHIVED, not left rotting in active rule files.
- Signs a rule is stale: never triggered in 10+ sessions, references deprecated tech, contradicts newer rules.
- Archive process: move to `brain/03-knowledge/retired-rules.md` with date and reason for retirement.
- Never just delete — archived rules provide context for WHY things changed.
- Review for retirement during `/weekly` retrospectives. Rule count should stay stable, not only grow.

## Strategic Reference: Continuous Improvement
- **Kaizen** (improvement): small, continuous improvements compound exponentially. 1% better each session = 37x better in a year.
- **PDCA with Study, not Check** (Deming): Plan -> Do -> STUDY -> Act. "Study" means deeply understand WHY results differ from expectations. "Check" is just pass/fail — insufficient.
- The goal is not perfection. The goal is a system that RELIABLY gets better over time.
- If the system isn't improving, the feedback loops are broken. Fix the loops, not the symptoms.
