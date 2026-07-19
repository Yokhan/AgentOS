---
name: codex-update-deps
description: "Safely update dependencies, resolve advisories, verify package availability, read changelogs, run tests, and document migration risks. Trigger on dependency updates, npm audit, CVE, or outdated packages."
---

# Codex Update Dependencies

Dependency and advisory data is time-sensitive. Verify against official registries or vendor advisories before recommending versions.

## Process

1. Inventory manifests and lockfiles.
2. Identify security versus routine updates.
3. For advisories, verify current official data.
4. Prefer patch/minor updates first.
5. For majors, read migration guides and plan changes.
6. Run tests and document remaining risk.
