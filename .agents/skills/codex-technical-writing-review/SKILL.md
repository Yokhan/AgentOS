---
name: codex-technical-writing-review
description: "Independently review technical documentation for product accuracy, executable procedures, information architecture, environment fit, and lifecycle ownership."
---

# Codex Technical Writing Review

This is an acceptance skill, not a generation pass. Read the shared writing SOT,
technical profile, reference registry, editorial board, and the product sources
named by the writing contract.

## Required Lanes

1. **Accuracy**: compare facts, names, defaults, limits, errors, and versions with
   code, schemas, interfaces, tests, and observed product behavior.
2. **Procedure**: execute the primary path in the stated OS and shell; check
   prerequisites, expected output, failure, recovery, and destructive boundaries.
3. **Architecture**: verify the chosen tutorial/how-to/reference/explanation,
   runbook, migration, or troubleshooting job and its navigation/lifecycle owner.
4. **Language**: verify terminology, normative force, code formatting,
   accessibility, localization readiness, and unambiguous steps.

Use `docs_researcher` for current official sources, `tester` for procedure design,
and `reviewer` for independent findings when fan-out has material value. Add
`$codex-api-contract`, `$codex-security-audit`, or `$codex-migrate` only when the
document crosses those contracts.

Report findings first by severity with evidence, user impact, violated contract,
and correction. State what was executed, what was only inspected, and residual
risk. Never accept generated names, a prose self-check, or unrun commands as proof.
