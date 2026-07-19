---
name: codex-security-audit
description: "Security review and patch workflow for vulnerabilities, secrets, auth, permissions, injection, XSS, SSRF, CVEs, data exposure, or suspicious dependencies. Treat as high-risk by default."
---

# Codex Security Audit

Read:

- `.claude/library/process/risk-classification.md`
- `.claude/skills/security-audit/SKILL.md`
- `.claude/library/technical/error-handling.md`

## Process

1. Identify assets, actors, trust boundaries, and data exposure.
2. Classify severity and exploitability.
3. Check official advisories or primary sources for current CVE/package data.
4. Patch narrowly; no unrelated refactors.
5. Add tests proving the exploit path is closed.
6. Run security-relevant checks.
7. State remaining exposure, rotation needs, and deployment steps.
