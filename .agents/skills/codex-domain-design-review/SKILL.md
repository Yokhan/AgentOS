---
name: codex-domain-design-review
description: "Review UI, UX, product design, visual systems, accessibility, responsive behavior, and Figma/design-system decisions. Use for design audits, visual QA, screenshots, or interface critique."
---

# Codex Domain Design Review

Use this for critique and audit. Use `$codex-design-workflow` when creating or changing UI.

Read `.claude/skills/domain-design-review/SKILL.md` and `.claude/library/domain/domain-design-pipeline.md` when deeper evidence is needed.
Read `.agents/skills/codex-design-workflow/references/design-command-modes.md` for M+ critique, register selection, hardening, polish, or distillation.

## Review Focus

1. User task fit, app-specific KPI impact, and the chosen design register.
2. Subtraction audit: Keep, Remove, Collapse, Move, then Add only after subtraction.
3. Token and component reuse.
4. Layout consistency across target viewports.
5. Interaction states: default, hover, active, focus, disabled, loading, error, empty.
6. Accessibility: contrast, focus visibility, labels, keyboard flow.
7. Visual defects: overlap, overflow, one-note palette, unstable dimensions.

Lead with concrete defects, affected screens/components, and the smallest useful fix.

## Critique Order

Inspect the actual UI, screenshot, or flow before reading deterministic validator output when possible. Automated findings are evidence, not the design verdict.

Always distinguish:

- Product register: task speed, clarity, error reduction, support load, retention, activation, conversion, or other app KPI.
- Brand register: memorability, trust, offer clarity, conversion, loyalty, and proof.

End M+ reviews with the next useful command mode: `distill`, `harden`, `polish`, `adapt`, `clarify`, `typeset/layout`, or `craft`.
