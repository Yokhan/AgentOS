---
name: codex-strategic-review
description: "Review plans, decisions, roadmaps, or execution against strategic thinking: user victory, constraints, reversibility, tradeoffs, sequencing, and local versus global optimization."
---

# Codex Strategic Review

Read `.claude/skills/strategic-review/SKILL.md` for the full checklist when needed.

## Process

1. State the real objective.
2. Identify the product user, app-specific business outcome, constraint, and success metric.
3. Check whether the plan optimizes the user/business outcome or local task completion.
4. Compare at least one alternative.
5. Call out technical work that is not tied to user experience, revenue, loyalty, retention, conversion, activation, support load, or another KPI.
6. Call out irreversible choices and weak assumptions.
7. Check for sycophancy: do not accept the user's proposed path if it lowers evidence quality, safety, privacy, product outcome, or app-specific KPI.
8. Check for fake completion risk: name what evidence must exist before the work can be called done.
9. Apply the TRIZ contradiction gate when constraints conflict: phrase "need X without causing Y", list existing resources, try separation by time/place/scope/mode/user segment, and prefer an ideal final result over a weak compromise.
10. Apply the Sun Tzu / stratagem terrain check for competitive strategy: map terrain, alternatives, competitors, constraints, center of gravity, timing, asymmetry, and favorable ground; prefer winning without direct confrontation; reject deception, dark patterns, or user-hostile manipulation.
11. For marketing/GTM work, include marketer lens: ICP/audience, positioning, offer clarity, funnel/buyer journey, channel/distribution plan, proof, CAC/LTV/ROAS/conversion measurement, and ethical risk.
12. Apply the Ilyakhov plan reality check for client-facing plans: plan after understanding, first useful iteration, next verifiable checkpoint, explicit replan when assumptions break, and no hidden budget or effort drift.
13. Check progressive JPEG delivery: first useful view, next sharpened evidence layer, rough edges, and replan trigger.
14. Check progressive JPEG implementation: known future product capabilities should have an end-state skeleton and 1% callable contracts instead of being absent or replaced by legacy harness proof.
15. Check object readiness: if no final plan exists, recommend a plan gate before implementation; if a 1% object exists, review inventory completeness before detail depth.
16. Check progressive layer replacement: wrong earlier iterations, obsolete stubs, disabled branches, stale flags, skipped tests, and release-only harnesses should be deleted, replaced, or time-boxed as migration scaffolding with a removal condition.
17. Check project-slice reporting: tagged working documents need fresh `PROGRESSIVE_STATUS` headers and a current `node scripts/progressive-status.js` slice before closeout.
18. Recommend the next smallest valuable move.
19. After repeated repair, compatibility-only scaffolding, architecture drift,
    or a proposed breaking rewrite, use `$codex-change-strategy`. Compare
    destination and transition alternatives against protected contracts, total cost,
    objective evidence, and the client approval boundary.

For client-facing plans, status, replans, and closeouts, follow
`.claude/library/process/client-executor-contract.md`.
