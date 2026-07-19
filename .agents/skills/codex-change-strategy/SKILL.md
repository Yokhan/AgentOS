---
name: codex-change-strategy
description: "Choose a bounded destination and safe transition using evidence, protected contracts, hard gates, total cost, and an approved change envelope. Use before the first patch when reading reveals a system mismatch, or after the repeated-repair fallback breaker, compatibility-only shim, architecture drift, stale-path test, sunk-cost behavior, or proposed breaking change."
---

# Codex Change Strategy

Read `.claude/library/process/change-strategy-gate.md` as the policy SOT.

## Workflow

1. During initial reading, run a bounded repair-path check over the affected
   path and direct consumers. Look for causal final-plan, SOT/owner, state,
   duplicate-path, compatibility-layer, or protected-boundary evidence. A
   discovered system mismatch triggers this skill before the first patch. A
   first isolated leaf defect stays in the normal workflow only when the scan
   supports that classification; a second failed repair is the fallback breaker.
2. If research changed pipeline, risk, or approval authority, rerun the semantic
   router once with a concise discovery record and replace stale route state.
   Treat `blockEdits` as pending only until the decision validator passes; then
   update the same route with `--decision-file` and resume its original pipeline.
3. Record the acceptance/falsifier ID and failed intervention evidence when the
   trigger is repeated failure; record repository/runtime evidence for a
   pre-repair architecture trigger.
4. Classify `greenfield|evolving|production|unknown` from deployments, users,
   data, and consumers, not repository age.
5. Inventory protected boundaries with owner, SOT, and impact.
6. Compare destination `repair|bounded-replace|retire-remove` and transition
   `direct-swap|staged-swap|versioned-coexistence|expand-migrate-contract`.
7. Eliminate options whose product, data, security, contract, verification, or
   recovery constraint is `fail` or safety-relevant `unknown`.
8. Compare viable options with objective evidence on a common baseline. Estimated values are
   forecasts, not advantages; performance needs a comparable measurement and
   maintainability cannot rely only on LOC.
9. Load API, data, or external-dependency compatibility profiles only when the
   affected boundaries require them.
10. Continue only inside a current approved change envelope. Ask the user when a
   material behavior, business outcome/KPI, contract, data, ownership, risk, release, downtime, cost,
   timeline, security, or irreversible-state limit changes.
11. Remove the superseded path or time-box transition scaffolding with owner,
    removal condition, recovery, and absence check.

## Record And Validate

Use the active orchestrator artifact. Do not create competing task state:

- AgentOS Gate when AgentOS is present;
- existing Spec/Plan/Tasks artifact when present;
- `tasks/current.md` or optional `tasks/change-strategy.json` for parent Codex;
- response-only decision record for read-only work.

Validate JSON decisions with:

```bash
node scripts/validate-change-strategy.js tasks/change-strategy.json
```

Report trigger, posture, protected boundaries, destination x transition,
evidence levels, recommendation, rejected alternative, approved envelope or
required decision, cleanup/recovery, and regression guard. Notify the user even
when the existing envelope allows automatic continuation.

The validator proves decision completeness, not architectural correctness.
