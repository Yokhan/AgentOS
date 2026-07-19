# Ilyakhov Planning And Client Communication Principles

Source: user-provided photos of pages from an Ilyakhov book. This note is a
paraphrase for internal use, not a verbatim excerpt.

## Why This Matters For Agents

Agents often fail planning work in the same way people do:

- they give a plan before they understand the task;
- they treat the plan as a promise instead of a forecast;
- they hide uncertainty behind vague words;
- they keep working when the plan is already detached from reality;
- they report effort instead of a useful result.

The useful principle is simple: a plan is a coordination tool. It should help
the client make decisions, see progress, and regain control when reality
changes.

In this template, "client" means the user, product owner, downstream team, or
any person who depends on the agent's result.

## Core Principles

### Plan After Understanding

Before giving a plan, understand:

- what result the client needs;
- who will use the result;
- what decision or business outcome it supports;
- what materials, access, people, and approvals are needed;
- which tasks depend on each other;
- which tasks can run in parallel;
- what external factors can delay the work;
- which deadline is real and why.

Do not plan from optimism. Plan from constraints.

### A Plan Is Not Reality

The plan is a current model of the work, not the work itself. It will become
wrong when facts change.

Good behavior:

- compare the plan with reality often;
- name the first sign that the plan is drifting;
- update the plan before the client has to ask;
- explain what changed and what decision is needed.

Bad behavior:

- keep working silently after the plan is broken;
- hope the delay disappears;
- report only that "work continues";
- surprise the client at the end.

### Prefer Useful Iterations

Classic staged planning can produce a result only at the end. Iterative planning
should produce a useful result at each step.

An iteration is good when:

- the client can use or evaluate the result immediately;
- the result can stand alone if the project stops;
- the next iteration becomes clearer because of real feedback;
- payment, approval, or risk can be tied to the delivered piece.

An iteration is bad when it is only a hidden internal checkpoint. "Finished
research", "prepared draft structure", or "started implementation" may be
useful status, but they are not product value by themselves.

### Explain The Value Of Iterations

Clients may prefer a single final deadline because it feels simpler. Explain
why iterations help:

- they reduce the risk of spending weeks on the wrong direction;
- they let the client see benefit earlier;
- they create decision points instead of a final surprise;
- they make scope, budget, and priority easier to adjust;
- they protect the project from one large all-or-nothing failure.

Do not sell iterations as speed. Iterations may make the calendar longer. Their
value is control, learning, lower risk, and earlier useful output.

### Use Concrete Time Language

Avoid vague time promises:

- "soon";
- "as quickly as possible";
- "we will see";
- "I do not know" as a final answer;
- "it is almost ready" without a check time.

Better:

- "I will return with the next check at 16:00.";
- "The next useful result is the route audit. I can show it today by 18:00.";
- "I cannot promise the final fix yet. I can promise a diagnosis checkpoint in 40 minutes.";
- "The deadline depends on access to X. If access is not ready by Tuesday 12:00, the earliest delivery moves to Thursday.";

The agent should not invent certainty. If the final date is unknown, give the
next verifiable checkpoint.

### If The Plan Breaks, Replan Explicitly

When the plan becomes unrealistic, choose one of these moves:

- ask for help or delegate a bounded part;
- renegotiate deadline, scope, or quality bar;
- reduce scope while preserving the real user outcome;
- pause work until the missing input arrives;
- stop the project if the useful result is no longer realistic;
- focus on the critical path and drop optional work;
- rest or reset if fatigue is now the source of errors.

Do not present a broken plan as still valid.

### No Hidden Budget Or Effort Drift

If the task starts consuming more time, money, or attention than expected, say
so before the client discovers it.

Useful report shape:

- what changed;
- what it costs in time, money, risk, or quality;
- what options exist now;
- what you recommend and why.

Never let the client keep paying for motion when the work no longer moves the
product toward a useful result.

## Agent Adaptation

### Planning Output Shape

Use this shape for non-trivial plans:

```text
Goal:
What useful result the user gets.

Current constraints:
Dependencies, unknowns, deadline reason, external risks.

Iteration 1:
The first useful result, not just internal preparation.

Checkpoint:
When I will show evidence or ask for a decision.

If this slips:
What changes first: scope, deadline, help, or plan.
```

### Status Update Shape

Use this when work is in progress:

```text
Current state:
What is true now.

Next visible result:
What the user will be able to inspect or decide from.

Time:
When I will return with evidence, not just "soon".

Risk:
What could move the plan and what I will do if it happens.
```

### Replan Shape

Use this when the old plan no longer matches reality:

```text
The old plan assumed:
...

Reality changed:
...

Impact:
...

Options:
1. Keep deadline, reduce scope.
2. Keep scope, move deadline.
3. Add help or split work.
4. Stop/pause because the result no longer makes sense.

Recommendation:
...
```

## Compatibility With This Template

### Strong Fit

These principles support existing template rules:

- `production-product-standard.md`: plan around real user and business outcome,
  not internal effort.
- `product-goal-loop.md`: keep final outcome, current step, dependencies, and
  risks visible.
- `plan-first.md`: plan before state-changing work, but keep plans reversible.
- `self-verification.md`: surface doubt and update confidence when facts change.
- `writing.md`: report from the reader's world, not from the agent's tool log.
- `codex-design-workflow`: iterations fit the existing idea of useful modes,
  rendered evidence, and current user job.

### Potential Conflicts

1. "Always give an exact date" can become false certainty.

Template-safe adaptation: give an exact next checkpoint when the final delivery
date is unknown. Pair it with confidence and dependencies.

2. Iterations can become an excuse for unfinished work.

Template-safe adaptation: an iteration must deliver a useful result or a real
decision point. Internal progress alone does not count.

3. Replanning can bypass user approval.

Template-safe adaptation: the agent may recommend a new plan, but state-changing
scope, deadline, or quality-bar changes need explicit user confirmation when
they affect the promised outcome.

4. "Ask for help" can conflict with tool and subagent permissions.

Template-safe adaptation: treat help as an option, not an automatic action.
Use subagents only when the active tool policy and user request allow it.

5. Vague words should not become a brittle ban-list.

Template-safe adaptation: do not hard-fail every use of "soon" or "as quickly
as possible" in prose. Enforce this in planning/status contexts where the user
needs a decision.

## Integration Recommendation

Superseded integration decision, 2026-07-05: keep this full note as cold
knowledge, but promote compact behavior shapes into `AGENTS.md`, `CLAUDE.md`,
shared strategic rules, and planning skills. The older recommendation to avoid
`AGENTS.md` still applies to the full note text, not to short hot-path examples
that the product owner explicitly requested.

Good next integration points:

1. Add a short "Plan Reality Check" section to
   `.claude/library/process/plan-first.md`.
2. Add "next verifiable checkpoint" language to
   `.claude/library/process/product-goal-loop.md`.
3. Add a communication example to `.claude/library/technical/writing.md` for
   slips, replans, and status reports.
4. Add a focused regression test only if planning/status wording becomes a
   repeated failure.

Do not create a broad validator yet. The rule is contextual: the same word can
be harmless in a narrative and harmful in a deadline promise.
