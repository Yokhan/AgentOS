---
name: codex-debug
description: "Systematically debug failures: reproduce, isolate, diagnose root cause, patch minimally, add regression coverage, verify, and log lessons. Trigger on bug, crash, failing test, regression, broken, почини, or не работает."
---

# Codex Debug

Read `.claude/skills/debug/SKILL.md` for deeper methodology when needed.

## Process

1. State the observed failure and expected behavior.
2. Reproduce the failure or explain why it cannot be reproduced.
3. Read the failing code path, callers, tests, recent history, and lessons. Run
   a bounded repair-path check over the affected path and direct consumers for
   causal boundary, ownership, duplicate-state, or obsolete-path evidence.
4. Diagnose root cause before editing. If reading reveals a qualifying system
   mismatch, invoke `$codex-change-strategy` before the first patch. Rerun the
   original route once only when the discovery changes pipeline, risk, or
   approval authority.
5. Patch the root cause with the smallest reasonable change toward the accepted
   final architecture, not automatically the smallest diff.
6. Add or update a regression test or smoke check.
7. Run relevant checks.
8. Log reusable bug patterns to `tasks/lessons.md`.

The second failed repair is a mandatory fallback circuit breaker, not the first
time architecture is considered. Stop local patching and use
`$codex-change-strategy`. Compare destination and transition alternatives; do
not present options without protected-contract and evidence analysis.
