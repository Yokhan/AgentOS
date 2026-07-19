# Brainstorm Protocol — Think Before You Plan

## When to Brainstorm

| Risk Level | Task Size | Brainstorm? |
|------------|-----------|-------------|
| LOW | Any | Skip |
| MEDIUM | XS-S | Skip |
| MEDIUM | M+ | Optional (recommended) |
| HIGH | Any | Recommended |
| CRITICAL | Any | Mandatory (exception: security patches — fix first, brainstorm later) |

Brainstorming happens BEFORE planning. It is Socratic exploration, not solution design.

## Phase 1: Problem Space (diverge)

Ask and answer these questions:
1. **What is the REAL problem?** Not the stated task — the underlying need the user is trying to solve.
2. **What are 3 fundamentally different approaches?** Force yourself to find at least 3. If you can only think of 1, you haven't explored enough.
3. **What is each approach's CENTER OF GRAVITY?** The one thing that makes it work or fail.
4. **What constraints exist that are NOT mentioned in the task?** Performance, backward compatibility, team familiarity, deployment constraints.
5. **What has been tried before?** Check lessons.md, git log, brain/04-decisions/.

## Phase 2: Trade-off Analysis (converge)

For each approach from Phase 1:

| Approach | Effort | Risk | Reversibility | Extensibility | Dependencies |
|----------|--------|------|---------------|---------------|--------------|
| A: ... | L/M/H | L/M/H | easy/hard/none | good/limited | count |
| B: ... | ... | ... | ... | ... | ... |
| C: ... | ... | ... | ... | ... | ... |

Score each dimension. No approach is perfect — trade-offs are the point.

## Phase 3: Selection

- State which approach you **recommend** and **WHY** (1-2 sentences)
- State which approach you **rejected** and **WHY** (1 sentence each)
- If no clear winner: present to user with your recommendation

## Output Format

Save to `tasks/current.md` under `## Brainstorm` (before `## Plan`):

```markdown
## Brainstorm

### Problem
[1-2 sentences: the REAL problem, not the stated task]

### Approaches Considered
1. **[Approach A]**: [1 sentence] — Effort: [L/M/H], Risk: [L/M/H]
2. **[Approach B]**: [1 sentence] — Effort: [L/M/H], Risk: [L/M/H]
3. **[Approach C]**: [1 sentence] — Effort: [L/M/H], Risk: [L/M/H]

### Selected: [A/B/C]
**Why**: [1-2 sentences — the decisive factor]
**Why not [rejected 1]**: [1 sentence]
**Why not [rejected 2]**: [1 sentence]
```

## Rules

1. Brainstorm is THINKING, not coding. No file paths, no code snippets, no implementation details.
2. **Minimum 3 approaches.** If you can only think of 1, you haven't explored enough.
3. **"The obvious approach" must be challenged.** Why is it obvious? Is it actually best? Or just familiar?
4. Brainstorm output feeds into the Plan. Plan references brainstorm: "Selected approach B from brainstorm because..."
5. **Time budget**: 2-5 minutes of agent time (5-15 tool calls for research). If longer, the problem needs decomposition first.
6. Brainstorm is NOT a plan. Don't list files, don't estimate complexity, don't define implementation order. That's planning.

## Anti-Patterns

- **One-approach brainstorm**: "I'll just do X" is not brainstorming. Force alternatives.
- **Analysis paralysis**: 3 approaches is the target. 5+ means you're avoiding a decision.
- **Premature detail**: "I'll use a HashMap with custom comparator" — too detailed. Stay at approach level.
- **Ignoring the rejected**: If you can't articulate why alternatives were rejected, you haven't compared them.

## Integration

- Pipeline: feature.md includes optional Brainstorm step between Research and Plan
- Risk: brainstorm.md is triggered by risk level (see risk-classification.md)
- Plan: plan-first.md quality gate criterion #9 ensures plans describe WHAT not HOW — brainstorm is where the HOW is explored
