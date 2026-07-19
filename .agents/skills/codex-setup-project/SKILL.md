---
name: codex-setup-project
description: "Configure a generated project for Codex and Claude coexistence: detect stack, update AGENTS.md, PROJECT_SPEC, tooling, scripts, registry, hooks, skills, and verification. Trigger on setup project or настрой проект."
---

# Codex Setup Project

Use this for generated projects after bootstrap.

## Process

1. Gather stack, project type, name, and optional integrations.
2. Run or update project scan: `bash scripts/scan-project.sh`.
3. Fill `PROJECT_SPEC.md`, `_reference/tool-registry.md`, and stack docs.
4. Configure formatter, lint, typecheck, tests, and setup commands.
5. Keep `AGENTS.md` under the Codex size budget.
6. Preserve shared rules in `.claude/library/`.
7. Add project-specific Codex skills under `.agents/skills/project-*`.
8. Run relevant validation.

Do not hardcode model, effort, approval, or sandbox in project `.codex/config.toml`.
