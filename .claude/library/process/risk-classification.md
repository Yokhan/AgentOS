# Risk Classification Protocol

Every task gets a risk level BEFORE planning begins. Risk is independent of size — a one-line auth fix is HIGH risk despite being XS size.

## Classification Matrix

| Risk Level | Criteria | Examples |
|------------|----------|----------|
| **LOW** | Leaf code, no shared deps, easily reversible, test coverage >60% | New feature in isolated module, config change, doc update, CSS tweak |
| **MEDIUM** | Cross-module, new dependency, public API touched, test coverage 30-60% | Refactor touching 2+ modules, new external integration, new shared utility |
| **HIGH** | Shared/core code, auth/payments/PII, irreversible change, test coverage <30% | DB migration, auth flow change, payment logic, API contract change, core/ edits |
| **CRITICAL** | Data loss possible, security vulnerability, production-facing with no rollback | Security patch, data migration, credential rotation, crypto/session logic |

## How Risk Affects Process

| Dimension | LOW | MEDIUM | HIGH | CRITICAL |
|-----------|-----|--------|------|----------|
| Planning depth | Brief (S-style) | Standard plan | Full plan + Plan B | Full plan + Plan B + pre-mortem |
| Research depth | `get_context` brief | `get_context` normal | `get_context` full | full + security scan |
| Brainstorm | Skip | Optional (M+ recommended) | Recommended | Mandatory* |
| Verification gates | Gate 0 (lint) | Gate 1-2 | All 4 gates | All 4 + external reviewer |
| Review intensity | Approve fast | Standard review | Deep review (Opus) | Deep review + user approval |
| Approval required | No | No | User approval of plan | User approval of plan AND diff |
| Test requirements | Existing pass | Unit tests | Unit + integration + edge cases | Unit + integration + regression + security |

*Exception: security patches skip brainstorm — fix the vulnerability first, explore alternatives after.

## Classification Rules

1. **Classify BEFORE sizing.** Risk and size are independent dimensions.
2. **When in doubt, classify ONE LEVEL HIGHER.** Downgrading is cheap. Missing a risk is expensive.
3. **Risk overrides** — these ALWAYS trigger minimum HIGH:
   - Auth, security, payments, health-related code
   - Shared/core module changes
   - Irreversible operations (data deletion, schema drops)
   - External-facing API contract changes
4. **State classification explicitly**: `Risk: MEDIUM (touches shared/utils, 2 consumers)`
5. **If risk changes during implementation** (discovered new blast radius), STOP and reclassify.

## Blast Radius Check (MEDIUM+ risk)

Before proceeding past planning:
- Run `bash scripts/blast-radius.sh <file>` for each modified shared file
- Run `bash scripts/import-graph.sh` if touching core/ or shared/
- Document blast radius in plan: `Affects: [N] downstream files in [M] modules`

## Risk Escalation Triggers

During implementation, escalate risk if you discover:
- More files affected than planned (+50% = escalate one level)
- Shared code modification not anticipated in plan
- Test coverage lower than expected for affected area
- External service dependency not documented in PROJECT_SPEC.md
- Any security-adjacent logic encountered unexpectedly

When escalating: update risk level in tasks/current.md, apply new ceremony level immediately.

## Integration Points

- `plan-first.md`: Plan template includes Risk field referencing this protocol
- `self-verification.md`: Verification gates triggered by risk level
- `implementer.md`: Agent classifies risk after sizing, before planning
- All pipelines: Step 0/1 includes risk classification
