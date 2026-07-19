---
name: codex-coverage
description: "Analyze test coverage, identify untested code paths, design focused tests, and avoid coverage theater. Trigger on coverage, untested, test gaps, missing tests, or quality gate failures."
---

# Codex Coverage

Read `.claude/skills/coverage/SKILL.md` and `.claude/library/technical/testing.md`.

## Process

1. Identify the behavior at risk, not just the percentage.
2. Map uncovered public paths and important edge cases.
3. Prioritize tests by blast radius and failure impact.
4. Add focused unit, integration, or E2E tests as appropriate.
5. Run relevant coverage commands.
6. Report remaining test gaps honestly.
