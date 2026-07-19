# Plan-First Protocol — Architecture Before Code

For product, design, auth, data, game, docs, deployment, template, or M+ work, read `.claude/library/process/product-goal-loop.md`, `.claude/library/product/production-product-standard.md`, and `.claude/library/process/client-executor-contract.md` before planning.

Plans, audits, and reports must use the language of the user's request.

## The Rule
For every task S+ size, the agent MUST produce a written plan with file structure and complexity estimate BEFORE writing any code.
Plans are saved to `tasks/current.md` under `## Plan` and tracked alongside the work.

Coding without a plan = bloated files, wrong structure, rework.

## When Planning is Mandatory

| Task Size | Planning Required | Detail Level |
|-----------|------------------|--------------|
| **XS** | No | Just do it |
| **S** | Brief | 3-5 lines: what files, what changes |
| **M** | Standard | Full plan (see template below) |
| **L** | Detailed | Full plan + user approval before coding |
| **XL** | Decompose first | Break into M-tasks, plan each one |

## Plan Template (M+ tasks)

Write this to `tasks/current.md` under `## Plan` BEFORE coding:

```markdown
## Plan

### Goal
[1 sentence: what this achieves for the user]

### Product Goal Link
- Final outcome: [from `tasks/goal.md` or inferred]
- Current step: [bounded step for this task]
- Quality bar preserved: [UX/security/privacy/performance/design/data/etc.]
- Out of scope for this step: [honest exclusions]

### Complexity Estimate
- Size: [XS/S/M/L/XL]
- Files to create: [count]
- Files to modify: [count]
- Estimated lines: [range]
- Risk: [LOW/MEDIUM/HIGH/CRITICAL — classify per risk-classification.md]

### File Architecture
[Directory tree of files to create/modify with purpose of each]

```
src/
  features/
    auth/
      auth.service.ts    — [NEW] login/logout logic
      auth.types.ts      — [NEW] AuthUser, AuthError types
      auth.test.ts       — [NEW] unit tests
      index.ts           — [MODIFY] add auth exports
```

### Implementation Order
1. [First file/step — why first]
2. [Second — depends on #1 because...]
3. [Third — ...]

### Boundaries
- Max file size: 375 lines. If any file approaches limit → split plan now, not later.
- Module boundary: [which module(s) this touches]
- Public API changes: [yes/no — if yes, document contract]

### Change Strategy (when triggered)
- Trigger: [second failed repair, compatibility shim, architecture drift,
  stale-path test, sunk cost, or planned breaking change]
- Project posture: [greenfield/evolving/production/unknown, with evidence]
- Protected contracts: [user behavior, data, API, security, release,
  project-owned overlays, or none]
- Destination: [repair, bounded-replace, retire-remove]
- Transition: [direct-swap, staged-swap, versioned-coexistence, expand-migrate-contract]
- Objective evidence: [common baseline; measured/observed/estimated/unknown]
- Approval boundary: [automatic internal change or client-owned tradeoff]
- Removal/migration plan: [superseded path, owner, condition, rollback]

When this section applies, follow
`.claude/library/process/change-strategy-gate.md` and record the decision in the
active orchestrator artifact. Optional `tasks/change-strategy.json` decisions
must pass `node scripts/validate-change-strategy.js tasks/change-strategy.json`
before the next state-changing patch.

### Risks & Mitigations
- [Risk 1] → [Mitigation]
- [Risk 2] → [Mitigation]

### Client Checkpoint
- Acceptance: [what the client can accept or reject]
- First useful result: [decision, evidence, or artifact; not only internal work]
- Next evidence point: [test, source, diff, screenshot, route, or review]
- If this slips: [scope, deadline, help, or plan change to propose]

### Progressive JPEG
- Current low-resolution view: [what the client can understand or inspect now]
- Next sharpened layer: [what will become clearer after the next check]
- Rough edges: [what is intentionally incomplete or uncertain now]
- Replan trigger: [which fact changes scope, deadline, quality, or path]
- Final object plan: [accepted object inventory, contracts, states, functions,
  dependencies, and production function; if missing, planning is the current
  step]
- Implementation shape: [known future capabilities included as 1% callable
  hooks/stubs/contracts, and which speculative capabilities are intentionally
  excluded]
- Replacement/cleanup: [superseded stubs, wrong iterations, disabled branches,
  old tests, stale flags, or release-only harnesses to delete/replace/migrate]
- Slice purpose: [real user victory and product purpose fulfilled end to end at
  this detail level through the accepted final path]
- Journey and mechanism: [entry -> action -> feedback -> useful outcome ->
  return; why this produces the intended result]
- KPI and evidence: [app-specific KPI link, observed product evidence, and the
  fact that would falsify the claim]
- Truth boundary: [rough edges and stubs; confirm the slice outcome does not
  depend on a stub]

Planning, research, architecture, scaffolding, migration, status, tests, debug
output, mocks, and inventories are enabling checkpoints, not product slices.
Use `$codex-progressive-jpeg-planner` and validate
`tasks/progressive-plan.json` before implementation.

### Plan B (mandatory for M+ tasks)
If the primary approach fails at step [N], the fallback is:
- [Alternative approach — what changes]
- [What can be reused from Plan A]
- [Signal to switch: what specific failure triggers Plan B]

For S tasks: mental Plan B is fine (no need to write it out).
For L/XL tasks: Plan B must be written and approved by user alongside Plan A.
```

## File Architecture Rules

### Before creating ANY file, answer:
1. **Does a similar file already exist?** Search with Grep/Glob first. Extend > duplicate.
2. **Will this file stay under 375 lines?** If not, split the responsibility NOW.
3. **Does it belong in this directory?** Check architecture rules (vertical slices, dependency direction).
4. **Is the naming consistent?** Match existing patterns in the project.

### Splitting heuristic:
- One concept = one file. If you're naming it `utils.ts` or `helpers.py`, you're avoiding the decision of WHERE it belongs.
- If a planned file has 3+ unrelated functions → it's 2-3 files.
- If a planned module has 5+ files → consider if it's actually 2 modules.

## Complexity Estimation

### Count before you code:
- **Files to touch**: 1-3 = safe, 4-7 = medium (plan carefully), 8+ = decompose
- **New dependencies**: each new import/package = +1 complexity point
- **Affected tests**: count tests that will need updating
- **Cross-module changes**: each module boundary crossed = +2 complexity points

### Estimate accuracy rule:
- Your first estimate is probably 50% too low (planning fallacy)
- If estimate says "M" but you feel unsure → it's "L"
- If touching shared/ or core/ → bump complexity one level

## Plan Tracking

### During implementation:
- Check off completed steps in the plan
- If the plan changes (new files needed, different structure) → UPDATE THE PLAN FIRST, then code
- If complexity exceeds estimate by >50% → STOP, reassess, inform user

### After implementation:
- Compare actual vs planned: files created, lines written, time spent
- If estimate was off by >2x → log to lessons.md WHY (improves future estimates)

## Planning Quality Gate (M+ tasks — must pass before coding begins)

A plan is NOT ready for execution until ALL applicable criteria are met.
If any criterion fails, refine the plan. Do not proceed to code.

| # | Criterion | Check | Fail Signal |
|---|-----------|-------|-------------|
| 1 | **Goal clarity** | Can you state the user's goal in ONE sentence without jargon? | If you need 2+ sentences, you don't understand the goal yet |
| 2 | **Scope boundary** | Are the files to modify/create explicitly listed? | "And maybe some others" = fail |
| 3 | **Dependency map** | Are imports, consumers, and cross-module effects documented? | No blast-radius for MEDIUM+ risk = fail |
| 4 | **Test scenarios** | Are happy path, edge cases, and error scenarios enumerated? | "Will add tests" without specifics = fail |
| 5 | **Risk classification** | Is risk level stated with justification? | Missing risk level = fail |
| 6 | **Size confidence** | Is estimate based on file count + line count, not gut feeling? | "Should be quick" = fail |
| 7 | **Reversibility** | Can this be reverted with `git revert`? If not, is rollback plan documented? | Irreversible change without rollback plan = fail |
| 8 | **Plan B exists** | Is there a concrete alternative if primary approach fails? | "We'll figure it out" = fail |
| 9 | **No premature code** | Does the plan describe WHAT and WHY, not HOW in code? | Code snippets in plan = premature |
| 10 | **Evidence before done** | Is the evidence needed for acceptance explicit? | "Done" can be claimed without a fresh test/check/source |
| 11 | **No sycophancy** | Does the plan challenge harmful shortcuts or weak assumptions? | User preference is accepted even when it lowers outcome, safety, or quality |
| 12 | **Progressive JPEG** | Is the first useful view, next sharpened layer, rough edge, and replan trigger explicit? | The client sees only internal work or a final surprise |
| 13 | **Change strategy** | After repeated repair or architecture drift, are destination, transition, protected contracts, evidence, approval, and cleanup explicit? | Another local patch proceeds because it changes fewer lines |

**Scoring**: 13/13 = proceed. 11-12/13 = proceed with noted gaps. <11/13 = refine before coding.

For L/XL tasks, add:
- [ ] User has approved the plan
- [ ] Decomposition into M-sized subtasks is complete
- [ ] Each subtask passes criteria 1-13 independently

## Test Scenario Templates (required in plan for M+ tasks)

Before writing any code, enumerate test scenarios in the plan. This is NOT about writing test code — it's about THINKING about what could go wrong.

For EACH implementation unit (service, component, endpoint), add to plan:

```markdown
### Test Scenarios: [unit name]

**Happy path:**
- [ ] [Input] → [Expected output/behavior]
- [ ] [Another normal case]

**Edge cases:**
- [ ] Empty/null input → [expected behavior]
- [ ] Maximum/boundary values → [expected behavior]

**Error scenarios:**
- [ ] [Specific failure condition] → [expected error + message]
- [ ] Network/IO failure → [expected degradation]

**Integration:**
- [ ] [Component A] + [Component B] → [expected interaction]
```

### Minimum requirements by risk:

| Risk Level | Happy Path | Edge Cases | Errors | Integration |
|------------|-----------|------------|--------|-------------|
| LOW | 1 | 1 | 0 | 0 |
| MEDIUM | 2 | 2 | 1 | 0 |
| HIGH | 2 | 3 | 2 | 1 |
| CRITICAL | 3 | 3 | 2 | 2 |

### Rules:
1. If you cannot enumerate edge cases, you don't understand the problem well enough. Go back to research.
2. Scenarios written in plan are COMMITMENTS — implementation is not done until all have passing tests.
3. Test scenarios become the TEST step's input — test-engineer implements tests from these scenarios.
