---
name: codex-test-rules
description: "Validate template rules, skills, hooks, AGENTS instructions, and regression fixtures. Trigger when changing rule files, skill files, validation scripts, or agent instructions."
---

# Codex Test Rules

Use this when changing template instruction infrastructure rather than application code.

## Checks

1. Run syntax checks for shell and JavaScript validators.
2. Validate `.agents/skills` with `node scripts/validate-codex-skills.js`.
3. Confirm `AGENTS.md` stays below 32KB.
4. Confirm setup and sync scripts include any new template-owned paths.
5. Add or update smoke checks in `scripts/test-template.sh` when the shipped payload changes.

When a rule change affects behavior, include a small regression fixture or a command that proves the rule is enforced.
