---
name: codex-technical-writing
description: "Plan, write, and maintain technical documentation as an informational or communication overlay grounded in code, schemas, versions, environments, and executable evidence."
---

# Codex Technical Writing

Use this skill for API and SDK guides, developer documentation, README files,
runbooks, migrations, troubleshooting, release notes, architecture decisions,
and other text whose truth depends on a technical product.

Read:

1. `.claude/library/technical/writing.md`
2. `.claude/library/technical/technical-writing-profile.md`
3. `.claude/library/technical/writing-reference-registry.json`
4. `.claude/library/technical/writing-editorial-board.md`

## Process

1. Select informational as the primary mode, or communication for an incident,
   release, or migration notice with a recipient action.
2. Identify the reader, target language, product version, operating system,
   shell, deployment shape, document kind, active SOTs, and verification path.
3. Resolve language, process, domain, and technical profiles separately. Russian
   output loads `.claude/library/technical/russian-writing-profile.md`; vendor and
   standards sources may control correctness, terminology, accessibility, and
   information architecture, never Russian voice or syntax.
   Load the Russian explanation profile for explanations and the correspondence
   profile for incidents, releases, migration notices, requests, and status.
4. Select only applicable reference profile IDs. Validate the registry with
   `node scripts/validate-writing-references.js` before relying on them.
5. Inspect code, schemas, interfaces, tests, and observed behavior. Resolve SOT
   conflicts instead of silently preferring existing prose.
6. Produce the smallest complete document that lets the reader finish or
   understand the production job. Headings and unexecuted commands are not a
   functional progressive slice.
7. Run commands in the declared environment, record expected results and recovery,
   then replace superseded procedures rather than preserving active alternatives.
8. Request `$codex-technical-writing-review` for independent acceptance on M+,
   public, operational, security-sensitive, or versioned documentation.

An external writing service is optional evidence, not a substitute for these
checks. Do not claim its result unless configured access and a successful
artifact-specific response are recorded.

Load `$codex-api-contract` when endpoint behavior or schemas are part of the
contract. Load current official product documentation when freshness matters.
