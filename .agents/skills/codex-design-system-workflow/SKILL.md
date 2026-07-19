---
name: codex-design-system-workflow
description: "Build, audit, or extend production design systems with foundations, atoms, molecules, organisms, templates, Storybook, token tables, composition traces, and rendered geometry checks. Use for design-system, tokens, Storybook, component library, or UI contract work."
---

# Codex Design System Workflow

Read:

- `.claude/library/domain/domain-design-system.md`
- `.claude/library/domain/domain-design-pipeline.md`
- `.agents/skills/codex-design-workflow/references/design-command-modes.md` when the system change affects register, mode selection, hardening, polish, or critique.
- `_reference/tool-registry.md`

## Required Flow

1. Foundations: confirm root `DESIGN.md` when present, choose product or brand register, then tokens for color, typography, spacing, radius, motion, layout, and control sizes.
2. Atoms: confirm primitive controls and states.
3. Molecules: compose from atoms and tokens only.
4. Organisms: compose from lower layers and expose a dependency trace.
5. Templates: define responsive layout and density rules.
6. Screen anatomy: every full screen starts with root frame, base background, independent background composition, content frame, and optional overlay layer before product components are placed.
7. Plan gate: broad component skeleton work requires the final component contract or a plan step first.
8. End-state skeleton: accepted future behavior is exposed through 1% callable slots, states, handlers, events, or feature flags when it belongs to the final component contract.
9. Replacement gate: remove or replace superseded variants, obsolete stories, stale feature flags, disabled controls, and release-only harnesses; keep only final-plan placeholders or time-boxed migration scaffolding.
10. Screens/stories: use real product data and navigation states.
11. Verification: Storybook/browser screenshots plus computed-style and bounding-box checks.

## No Raw Values

If a needed value has no token, stop and add/request the token. Do not invent local values inside larger components.

Root `DESIGN.md` is a project-owned visual context file. Update it when visual direction, token meaning, component behavior, or guardrails change; do not use template sync to overwrite an existing project `DESIGN.md`.

## Required Stories

- Foundations tables: typography, spacing, radius, motion, layout.
- Atom states: default, hover, active, focus, disabled, loading, error, empty where applicable.
- Atom isolation: stories show the atom itself without unrelated frames, labels, or decorative wrappers unless the wrapper is part of the atom contract.
- Molecule composition traces.
- Organism and template responsive examples.
- Screen anatomy tables for full-page templates and screens.
- Product forms, account/auth, empty/loading/error, service gateway, docs/help surfaces when relevant.
- End-state skeleton stories or notes for accepted future capabilities that are stubbed, feature-flagged, no-op, or dev-debug only.
- Superseded layer cleanup notes for removed variants/stories/flags and any temporary migration scaffold with its removal condition.

## Screen Anatomy Contract

Every screen, template, and full-page Storybook example must declare:

- Root frame: viewport/min-height, isolation, overflow, base surface, and base text color.
- Base background: flat fill, gradient, image/media slot, or another approved surface token.
- Background composition: decorative/media layer independent from content spacing.
- Content frame: safe-area, responsive padding, max-width/grid, column model, and allowed organisms.
- Overlay layer: modals, drawers, toasts, sticky actions, only when the scenario needs them.

Bounded vs edge-to-edge rule:

- Visible bounded surfaces (glass, cards, panels, modals, framed media) own internal padding, radius, border/effect tokens, and content-slot rules.
- Edge-to-edge content sections use the page grid/content frame directly and do not get fake card padding.
- If a screen cannot name these layers, stop before styling. Do not place headers, heroes, forms, service cards, or docs content directly into a naked `main` or story shell.
