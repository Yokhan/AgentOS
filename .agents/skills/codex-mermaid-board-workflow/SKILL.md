---
name: codex-mermaid-board-workflow
description: "Create and maintain Mermaid control boards, architecture maps, delivery maps, risk maps, and project dashboards with .mmd files as the source of truth."
---

# Codex Mermaid Board Workflow

Use this skill when a task asks for Mermaid, diagrams, control boards, project maps, architecture maps, roadmap boards, audit boards, or visual project control.

## Process

1. State the board purpose: decision, control, explanation, audit, or handoff.
2. Pick one primary diagram type:
   - `flowchart LR` for stakeholder/control boards.
   - `flowchart TB` for lifecycle, state, and gate maps.
   - `sequenceDiagram` only for interaction timing.
   - `gantt` only for date-bound schedules.
3. Keep the `.mmd` file the source of truth. Do not make screenshots or visual exports canonical.
4. Model the board as visible entities: source, surface, action, gate, metric, owner, risk, decision.
5. Use semantic class names such as `source`, `surface`, `work`, `gate`, `metric`, `risk`, `done`, `waiting`.
6. Put stakeholder comments in separate visible comment cards near the group they explain. Avoid decorative edges.
7. Split large maps into submaps before readability fails. Link submaps from the main board.
8. For large boards, provide a viewer or preview with pan, wheel zoom, fit, actual-size, and SVG export controls.
9. Verify syntax with Mermaid tooling when available; otherwise run a text check for balanced fences, IDs, and obvious parser hazards.
10. Update the board after each material decision, not only at the end of a task.

## Board Gates

- Every board must have a title/purpose node.
- Every work lane must have a next action.
- Every gate must state the pass condition.
- Every metric must say what decision it changes.
- Every waiting node must name the missing input or owner.
- Avoid generic "phase 1 / phase 2" labels unless dates or gates make them meaningful.

## References

- `references/mermaid-board-patterns.md` - board patterns, syntax rules, and project-control conventions.
- `references/mermaid-control-board-template.mmd` - starter Mermaid control board.
- `references/mermaid-viewer-ux.md` - viewer and preview interaction rules.