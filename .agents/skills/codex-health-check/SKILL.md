---
name: codex-health-check
description: "Run project or template health checks: drift, validation, tests, security posture, dependency state, registry freshness, and release readiness."
---

# Codex Health Check

Use project-specific commands from `PROJECT_SPEC.md` and `_reference/tool-registry.md`.

Template baseline commands:

```bash
bash scripts/validate-template.sh
bash scripts/check-drift.sh
bash scripts/test-template.sh
bash scripts/sync-agents.sh
node scripts/validate-codex-skills.js
```

Report failures first with likely cause and next action.
