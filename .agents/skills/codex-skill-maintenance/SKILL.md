---
name: codex-skill-maintenance
description: "Create, update, validate, or audit Codex skills in .agents/skills with proper SKILL.md frontmatter, progressive disclosure, references, and template sync coverage."
---

# Codex Skill Maintenance

Official Codex repo-scoped skills live in `.agents/skills`.

## Process

1. Name skills with lowercase hyphen-case.
2. Put required `name` and `description` in `SKILL.md` frontmatter.
3. Keep `SKILL.md` short and move long catalogs to `references/`.
4. Add `agents/openai.yaml` with a default prompt mentioning `$skill-name`.
5. Avoid project-level model, effort, approval, and sandbox defaults.
6. Run `node scripts/validate-codex-skills.js`.
