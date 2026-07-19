# Self-Verification Protocol (MANDATORY)

This protocol is NOT optional. Skipping it is a system failure.

## Graduated Verification (scales with task size)

Gates are NOT one-size-fits-all. Match verification intensity to task risk:

| Task Size | Gates Required | Details |
|-----------|---------------|---------|
| **XS** | Gate 0 only | Typecheck/lint pass. No manual gates. |
| **S** | Gate 1 | Intent check: "Does this match the request?" |
| **M** | Gate 1 + Gate 2 | Intent check + Red-team own code. Confidence declaration. |
| **L** | All 4 gates | Full Doubt Protocol. User checkpoint at mid-build. |
| **XL** | All 4 gates + pre-mortem | Plus: reviewer agent mandatory. User approval at 3 checkpoints. |

**Risk-based overrides** (see `.claude/library/process/risk-classification.md` for full protocol):
- **HIGH risk** → all 4 gates mandatory, regardless of task size
- **CRITICAL risk** → all 4 gates + external reviewer + user approval of diff
- Risk classification drives gates, not just task size. A one-line auth fix (XS) at HIGH risk gets full 4 gates.

**Does NOT trigger** (no gates needed):
- Answering questions without code changes
- Single-file typo/formatting fixes (XS with no risk override)
- Adding tests without changing production code
- Documentation-only changes

## The Doubt Protocol (3 mandatory steps)

### Step 1: Devil's Advocate (Attack your own solution)

Before presenting ANY implementation, explicitly answer:
1. "What is the WEAKEST part of this solution?"
2. "If I were reviewing this as a SKEPTIC, what would I challenge?"
3. "What assumption am I making that could be WRONG?"
4. "Is there a SIMPLER approach I dismissed too quickly? Why did I dismiss it?"

If you cannot answer #1 — you haven't thought hard enough. EVERY non-trivial solution has a weak point. "None" is never an acceptable answer.

### Step 2: Commander's Intent Re-check

- Re-read the ORIGINAL request (not your plan, not your interpretation — the user's actual words)
- State: "The user's actual goal is: [X]"
- State: "My solution achieves this by: [Y]"
- State: "My solution could FAIL to achieve this if: [Z]"
- If Z is plausible → STOP. Reconsider before presenting.

### Step 3: Confidence Declaration (REQUIRED in output)

Every non-trivial response MUST end with:

```
VERIFICATION:
- Approach: [1-sentence summary of what I did]
- Confidence: [HIGH / MEDIUM / LOW]
- Doubt: [what I'm least sure about — NEVER "none"]
- Alternative considered: [what else I could have done]
- Risk: [what could go wrong with this approach]
```

Interpretation:
- **LOW** → Do NOT implement. Present 2-3 options with trade-offs. Let user decide.
- **MEDIUM** → Implement, but FLAG uncertainty explicitly. "I chose X but Y might be better because..."
- **HIGH** → Proceed. But: if you're HIGH on everything, you're not doubting enough.

Calibration rule: if you haven't explicitly considered and rejected at least one alternative, your confidence is MEDIUM at best, never HIGH.

## The Sunk Cost Test

Before EVERY response where you've already started implementing:

> "If I had NOT already written this code, would I choose this exact approach?"

- YES → Continue.
- NO or PROBABLY NOT → STOP. Discard what you wrote. Restart with the approach you'd actually choose.

The code you've already written has ZERO value if it's the wrong approach. Throwing it away is free. Defending it is expensive.

## Sycophancy Circuit Breaker

When the user points out a flaw and your reaction is "you're right!":
1. That means you ALREADY KNEW but didn't surface it
2. This is an ATTENTION FAILURE, not a knowledge gap
3. Log to tasks/lessons.md: what the flaw was, why you didn't catch it, what verification step would have caught it
4. Next time: the same category of flaw must be caught by YOUR verification, not the user's

Goal: zero "you're right!" moments. Every flaw should be surfaced by the agent BEFORE the user has to point it out.

## Error Recovery Protocol ("I'm stuck")

When confidence is LOW or you're blocked after 2+ attempts:

1. **STOP coding**. Do not retry the same approach.
2. **State what you know**: "I attempted X. It failed because Y. I've tried Z alternatives."
3. **State what you don't know**: "I'm uncertain about [specific gap]."
4. **Present options** (max 3): each with effort estimate and risk level
5. **Ask the user** to choose — or suggest what information would unblock you
6. **Log to tasks/lessons.md** after resolution: what was the block, what resolved it

Never spin silently. The user seeing "I'm stuck on X, here are my options" is 100x better than the agent producing wrong code on attempt #4.

See also: `.claude/library/conflict/conflict-resolution.md` — Escalation Protocol, Confidence-Based Escalation.

## Reuse Gate (M+ tasks, in addition to Doubt Protocol)

Before presenting implementation:
- [ ] Did I search for existing utilities before creating new ones? (grep + tool-registry)
- [ ] Are there similar functions elsewhere I should consolidate?
- [ ] New shared utilities registered in `_reference/tool-registry.md`?
- [ ] If I created a helper — is it genuinely needed in 2+ places, or should it be inline?

If any answer is "no" or "didn't check" → go back and check before presenting.

## Anti-Patterns This Prevents

| Bias | How it manifests | How this protocol stops it |
|------|-----------------|---------------------------|
| Commitment bias | Defending approach A after choosing it | Sunk Cost Test forces re-evaluation |
| Completion bias | Rushing to "done" | Verification gate sits between done and output |
| Self-sycophancy | Agreeing with own previous output | Devil's Advocate forces attack on own solution |
| Tunnel vision | Not considering alternatives | Confidence requires rejected alternative |
| Authority bias | "Best practice says..." | Critical-thinking.md evidence hierarchy |

## References
- `.claude/library/meta/critical-thinking.md` — evidence hierarchy, red flags
- `.claude/library/meta/strategic-thinking.md` — OODA loop, Commander's Intent
- `.claude/library/conflict/conflict-resolution.md` — confidence-based escalation
- `.claude/library/process/self-improvement.md` — sycophancy failure tracking (must align with circuit breaker above)
- `.claude/library/meta/critical-thinking.md` — evidence hierarchy prevents overconfident recommendations
