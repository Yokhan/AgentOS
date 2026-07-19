---
name: codex-figma-workflow
description: "Use Figma MCP safely for design creation, editing, syncing, screenshots, Code Connect, and design-system composition. Trigger for Figma URLs, Figma writes, mockups, design system work, or capture-to-Figma tasks."
---

# Codex Figma Workflow

## Before Writing

1. Load figma-use guidance when available.
2. Search the design system before creating components.
3. Discover variables, text styles, and components.
4. Prefer imported instances and token bindings over raw shapes.

## Tool Choice

- Use `use_figma` for Figma writes and updates.
- Use `generate_figma_design` only for first-time web page capture.
- Use both for web app capture when the capture is a layout reference and `use_figma` builds the editable design.
- Use `get_screenshot` after structural changes.

## Validation

- Confirm layout mode on containers.
- Confirm tokens/styles are applied where available.
- Check 8 states for interactive components.
- Register reusable tokens/components in `_reference/tool-registry.md` when they become project assets.
