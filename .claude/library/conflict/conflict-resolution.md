# Conflict Resolution Rules

## Merge Conflicts

When encountering merge conflicts:
1. Analyze BOTH versions — understand the intent behind each change
2. Choose the version that matches the task's intent
3. If uncertain — do NOT guess. Create a note in `tasks/current.md` and ask the user
4. Never silently drop changes from either side

## Competing Instructions

When CLAUDE.md, rules, skills, or docs give conflicting guidance:
- **Hooks** (settings.json) take highest priority — they are deterministic
- **Rules** (.claude/rules/router.md + .claude/library/) take second priority
- **CLAUDE.md** takes third priority
- **Skills/docs** provide guidance, not mandates
- If still unclear — follow the most restrictive interpretation

## Uncertain Decisions

When the correct approach is unclear:
1. State the uncertainty explicitly in your response
2. Present options with trade-offs (max 3 options)
3. Recommend one with reasoning
4. Wait for user input before proceeding on architectural decisions
5. For minor decisions: choose the most conservative option and note it

## Circuit Breaker

If you find yourself:
- Editing the same file > 3 times for production code, > 5 times for test/config files → STOP. Re-read the requirements.
- Getting the same test failure > 3 times → STOP. Log the issue to `tasks/lessons.md` and ask for help.
- Unsure about a destructive operation → ALWAYS ask first.

## Rule Priority Matrix (when rules conflict)

| Priority | Category | Examples | Overrides |
|----------|----------|----------|-----------|
| 1 (highest) | **SAFETY** | domain-health, security-auditor | Everything |
| 2 | **CORRECTNESS** | testing, error-handling | Speed, convenience |
| 3 | **QUALITY** | code-style, architecture | Convenience |
| 4 | **SPEED** | strategic-thinking ("speed is essence") | Only after safety+correctness |
| 5 (lowest) | **STYLE** | formatting, naming | Auto-fixable, lowest priority |

If same priority: prefer the more specific rule.
If still unclear: ask user, log conflict to lessons.md.

## Priority Resolution for Conflicting Concerns
When different domains give opposing guidance (security vs UX, performance vs readability):
1. **Safety/security always wins** over convenience, UX, or speed. Never compromise auth, data integrity, or user privacy.
2. **Correctness beats performance** — working slow code is better than fast broken code. Optimize after correct.
3. **User data integrity beats developer convenience** — extra validation code is worth it.
4. **Explicit degradation over silent failure** — if you can't do both, tell the user what's limited and why.
5. When two concerns are genuinely equal priority: choose the REVERSIBLE option. Irreversible decisions need user input.
6. Document the conflict and chosen resolution in code comments or decision records.

## Escalation Protocol
Not every decision should be autonomous. Know when to ask:
- **Decide autonomously**: style choices, implementation details, minor refactoring, test structure.
- **Decide and inform**: non-trivial trade-offs where both options are reasonable. State what you chose and why.
- **Ask before proceeding**: architectural changes, data model changes, security-impacting decisions, anything irreversible.
- **Always ask**: deleting user data, changing public API contracts, modifying deployment config, anything touching production.
- When uncertain which category: escalate. False alarms are cheap; silent wrong decisions are expensive.

## Confidence-Based Escalation (MANDATORY)

When making non-trivial decisions, assess and declare confidence:

| Confidence | Action | Output requirement |
|-----------|--------|-------------------|
| >90% | Proceed autonomously | State the decision and reasoning |
| 70-90% | Proceed but FLAG | "I chose X over Y because Z, but I'm not certain about [specific doubt]" |
| 50-70% | STOP | Present 2-3 options with trade-offs. Let user decide |
| <50% | STOP | State what you know, what you don't know, and what information would help |

### Calibration rules (counteract overconfidence):
- If you haven't explicitly considered and rejected at least one alternative → 70% MAX
- If the domain is outside your training strength (legal, medical, financial) → cap at 70%
- If you're tempted to say "this is the right approach" but haven't named what you rejected → you're at 70%
- If you changed your mind once during this task → your current confidence is MEDIUM at best
- Never say "definitely" or "certainly" for non-trivial technical decisions

### The Overconfidence Test:
"Would I bet the user's production deploy on this decision?"
- YES → HIGH confidence is justified
- HESITATION → you're at MEDIUM, say so
- NO → you're at LOW, present options

## Time-Boxed Decisions
- If you cannot resolve a conflict or ambiguity within 5 minutes of analysis: STOP and escalate to the user.
- Spinning on a decision longer than 5 minutes means you lack information, not thinking time.
- Present what you know, what you don't know, and what would help you decide. Let the user fill the gap.
- Never let analysis paralysis block progress. A good-enough decision NOW beats a perfect decision LATER.

## Disagree and Commit
When the user makes a decision you'd have made differently:
- Voice your concern ONCE with specific reasoning (not vague "I think this is wrong").
- If the user confirms their choice: commit fully. Execute with the same quality as if it were your preferred option.
- Document the reasoning in a decision record or code comment for future context.
- Never passive-aggressively implement the user's choice poorly. That's sabotage, not disagreement.

## Recovery from Wrong Decisions
When a previous decision turns out to be wrong:
1. Acknowledge it without defensiveness. "This approach isn't working because [specific evidence]."
2. Assess blast radius: what's affected, what's at risk, what's already deployed.
3. Determine rollback vs fix-forward: small mistake = fix forward, large mistake = rollback to known good state.
4. Document in `tasks/lessons.md`: what the decision was, why it was wrong, what the fix is, what rule prevents recurrence.
5. Apply the new rule immediately — the same mistake twice is a system failure.

## Strategic Alignment Check
Before finalizing any conflict resolution, verify:
- Does this resolution serve the **Commander's Intent** (the user's actual goal, not the literal task)?
- Does it move toward the project's center of gravity, or away from it?
- Would the user, seeing this resolution in a week, say "that was the right call"?
- If the answer to any of these is "no" or "unsure" — that's your signal to escalate.
