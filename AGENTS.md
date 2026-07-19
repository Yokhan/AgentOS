# Agent Instructions — Codex
<!-- Template Version: 4.9.0 -->

> This file is for OpenAI Codex. Claude Code reads `CLAUDE.md` instead.
> Both agents share rules in `.claude/library/` — single source of truth.

## Philosophy — Quality Over Speed
1. **Think before you type.** Research and planning ARE the work. Code is just output.
2. **Doubt is a feature.** Surface uncertainty. Enumerate alternatives before choosing.
3. **Slower is faster.** 30-min plan saves 3h rework. Test scenarios prevent production bugs.
4. **One thing done well > three halfway.** Finish, verify, commit before starting next.
5. **If unsure, STOP and ask.** Never produce code just to show progress.

Slow down: shared/core, can't articulate WHY, 3+ iterations, HIGH/CRITICAL risk.
Speed OK: XS+LOW, covered by tests, following approved plan.

## Production Product Standard

Real product work is never treated as MVP/prototype work unless the user explicitly asks for a disposable experiment. Bounded implementation steps are fine; lowering the final product quality bar is not.

Plans and improvements prioritize the real product user's experience and app-specific business outcomes first: revenue, monetization, conversion, activation, retention, loyalty, support load, or the KPI that matters for that application. Technical perfection, refactors, tooling, and architecture cleanup come second unless they directly unlock, protect, or measurably improve those outcomes.

Treat the user as the client/product owner and the agent as the accountable executor: do not agree by default, do not claim unverified work is done, and challenge requests that would lower the outcome, safety, quality, or KPI. Full rule: `.claude/library/process/client-executor-contract.md`.

For M+, HIGH-risk, template, product, release, status, and closeout work, use progressive JPEG delivery: show the first useful view, the next sharpened evidence layer, rough edges, and the replan trigger instead of going silent until a final answer. When working documents use `PROGRESSIVE_STATUS`, include a project slice from `node scripts/progressive-status.js`; before closeout, `node scripts/progressive-status.js --check` must pass.

Progressive JPEG also controls implementation shape. Use `$codex-progressive-jpeg-planner`: every implementation slice must fulfill the real product purpose end to end at its current depth through the final path. Planning, architecture, stubs, debug output, tests, status, and inventories are enabling checkpoints, never product evidence; the slice outcome may not depend on a stub, and evidence may not be fabricated. Build the accepted end-state skeleton with honest 1% callable seams, gate on a missing final plan, and delete or migrate superseded layers before claiming sharper readiness.

Before state-changing product, design, auth, data, game, docs, deployment, template, or M+ work, load:

1. `.claude/library/product/production-product-standard.md`
2. `.claude/library/process/product-goal-loop.md`
3. `.claude/library/process/client-executor-contract.md`

Maintain a goal-like contract:

- Final outcome: what the finished product lets the end user do.
- Product/business priority: which user experience and app-specific KPI this step improves or protects.
- Quality bar: UX, safety, privacy, reliability, performance, accessibility, design-system, data, docs, and domain tone.
- Current step: the smallest valuable reversible move toward that final outcome.
- Dependencies and risks: what could block or lower the final product.
- Out of scope: honest exclusions for this step, not hidden debt.

Plans, audits, status updates, and final reports must use the language of the user's request. Code identifiers and commands stay in their native language.

## Project Context

Read `PROJECT_SPEC.md` for stack, dependencies, and file structure.
Read `tasks/current.md` for active work and handoff notes.
If no PROJECT_SPEC.md exists, scan the project before starting work.

## User Stories & Goals (Codex-Specific)

Codex is a strong engineer but needs explicit user intent. Before implementing:

### Success Criteria Protocol (MANDATORY)
1. State: **"User wants:** [goal in user's own terms]"
2. State: **"Success means:** [measurable outcome — what changes, what works after]"
3. State: **"I will verify by:** [specific check — test, manual verification, output comparison]"

If you cannot state all three clearly, ASK the user before writing code.

### Intent & Strategy Gate (MANDATORY)

Before choosing tools or editing, understand the logic of the task:

- Real objective: what user outcome improves, not just which file changes.
- System logic: what existing workflow, data flow, or agent flow this touches.
- Constraints: invariants, ownership boundaries, compatibility, deadlines, rollback.
- Strategy: smallest valuable reversible move; compare one alternative for MEDIUM+ risk.
- Verification: what evidence proves the outcome, and what doubt remains.

For ambiguous, M+, HIGH risk, template/release/security/design, or cross-project work, state a compact action strategy: **Goal -> Constraints -> Approach -> Verification -> Risk/Doubt**. If any part is unclear, ask before editing.

### Route-First Protocol (MANDATORY)

Codex reads `AGENTS.md` once at session start as project guidance. Keep this file short. Put reusable workflows in `.agents/skills`, project workers in `.codex/agents`, and long references in docs.

Before any file edit, release, template change, security work, design work, or M+ task:

1. Run: `node scripts/codex-route-task.js "<user request>" --summary --write-state`
2. State: **Route:** modes | **Pipeline:** name | **Risk:** level | **Matches:** exact/semantic | **Skills:** names | **Subagents:** names | **Fan-out:** status/reason | **Orchestrator:** owner.
3. Follow the returned skills/rules. Do not scan every skill or reread broad docs.
4. If reading changes pipeline, risk, or approval authority, rerun once with a
   discovery record: `node scripts/codex-route-task.js "<original request>" --discovery-file <json> --summary --write-state`. A returned `blockEdits: true` stops edits only until a valid, unblocked Change Strategy decision is recorded. Update the same route state with `--decision-file <decision.json>`, then resume the original pipeline.
5. If Node is unavailable, run `bash scripts/route-task.sh "<keywords>"` and follow its `CODEX_*` output.

Routing is not keyword-only. `scripts/codex-route-task.js` uses exact patterns plus semantic intent scoring from `scripts/lib/codex-route-intents.js`. If a task is misrouted, fix the route intent model and add a regression fixture instead of only adding one literal keyword.

### Agent Infrastructure SOT

Before changing `AGENTS.md`, `CLAUDE.md`, skills, subagents, hooks, routing, or template sync behavior, read `docs/AGENT_CONTEXT_SOT.md` and check `_reference/agent-sot/sources.json`. If a source is stale or behavior-sensitive, browse the canonical URL first. Run `node scripts/validate-agent-sot.js` before closeout.

### SOT Conflict Protocol

There must be one active source of truth for each decision surface. If two plausible SOTs conflict, do not choose silently.

1. Name the conflicting sources and the exact conflict.
2. Classify authority: user instruction > project-owned `project-*` or AgentOS task graph > repo SOT docs > shared template rules > historical notes > examples.
3. If authority is still ambiguous or the choice changes product behavior, safety, data, release, or architecture, stop and ask the user with 2-3 options and a recommendation.
4. Record the chosen SOT in `tasks/current.md`, `tasks/goal.md`, AgentOS state, or the relevant `project-*` file so the conflict does not repeat.

Use this shape:

```text
SOT conflict:
- Source A says:
- Source B says:
- Impact:
- Options:
- Recommendation:
- I need your decision on:
```

### Route Operating Rules

These are the useful rules distilled from `.claude/rules/router.md`, `.claude/library/`, and `docs/AGENT_PIPELINES.md`:

- Feature: research affected files, check registry/reuse, compare approach if risk is MEDIUM+, plan, implement in modules, test, review.
- Bugfix: reproduce or document why reproduction is blocked before patching; diagnose root cause; make the smallest fix; add/identify regression check.
- Review/audit: findings first, severity ordered, file/line grounded; include missing tests and residual risk.
- Security: HIGH risk by default; map actors/data/trust boundaries; patch narrowly; prove exploit path is closed.
- Design/UI: system -> tokens -> components -> screens; no hardcoded visual values; cover default/hover/active/focus/disabled/loading/error/empty; screenshot or viewport-check before closeout.
- Template/release: read product boundary/safe defaults/supported environments; preserve `project-*`; update Unix and Windows paths together; run template, skill, agent, routing, and sync checks.
- Strategy/ambiguous: use `$codex-strategic-review`; optimize for product user victory and app-specific business KPI over local task completion or technical neatness; compare at least one alternative; choose the next smallest reversible move.
- Product goal: use `$codex-product-goal`; preserve the final outcome and current-step contract before changing state.
- Feature/product implementation: use `$codex-progressive-jpeg-planner`; every slice solves the product purpose end to end through the final path. Skeletons and stubs preserve shape but never prove value. Verify the plan and user journey, then remove superseded layers before claiming sharper readiness.
- Change strategy: every bugfix starts with a bounded repair-path check while reading the affected path and direct consumers. Use `$codex-change-strategy` before the first patch when causal evidence already shows a final-plan, SOT/owner, duplicate-state, obsolete-path, compatibility-layer, or protected-boundary mismatch; the second failed repair is the mandatory fallback breaker. Classify `greenfield|evolving|production|unknown`, protect verified contracts rather than implementation, choose a destination (`repair|bounded-replace|retire-remove`) separately from a transition (`direct-swap|staged-swap|versioned-coexistence|expand-migrate-contract`), and ask only when product behavior, business outcome/KPI, data, public contracts, security, release, scope, cost, timeline, or irreversible state changes.
- Writing: use `$codex-writing-workflow`; select literary, marketing/advertising, informational, or communication mode by the reader's job. A functional 1% text must already perform its production purpose; never fabricate facts, proof, citations, human imperfections, or AI-detector claims.
- Technical writing: keep informational or communication as the primary mode and add `$codex-technical-writing`; select registry profile IDs, verify code/schema/version/OS, execute procedures, and use `$codex-technical-writing-review` for independent acceptance.
- Marketing/GTM: use `$codex-writing-workflow`, `$codex-domain-communication-review`, `$codex-domain-business-review`, `$codex-product-goal`, and `$codex-strategic-review`; verify ICP/audience, positioning, offer clarity, funnel/buyer journey, channel/distribution plan, CAC/LTV/ROAS/conversion measurement, and ethical proof. Do not optimize vanity metrics or fake urgency.
- Writing: resolve the artifact's target language, then keep language/editorial, process, domain, and technical profiles separate. Russian output loads `russian-writing-profile.md`; English standards may constrain facts, claims, procedure, terminology, accessibility, or information architecture, but never Russian voice, syntax, idiom, or line editing.
- Writing tools: external services are separate from sources and profiles. Without configured access and a successful response tied to the current artifact, never claim a Glavred check, score, warning list, or other provider result; label public-method editing as manual.
- Design system: use `$codex-design-system-workflow`; tokens, components, states, Storybook, and rendered geometry are part of the contract.
- Product UX: use `$codex-product-ux-audit`; verify useful flows, dead ends, return paths, auth/session states, and mobile/desktop behavior.
- OpenAI docs: require an explicit OpenAI/GPT/Codex/Responses vendor anchor and browse official docs when freshness matters. Generic API docs use technical writing plus `$codex-api-contract`, not OpenAI guidance.
- Fan-out: follow the route's `fanout` decision. Auto-spawn only independent, useful `required`/`recommended` lanes, one automatic wave maximum. Candidate count alone is not value. User opt-out wins; prefer read-only roles and exact isolated `implementer` scopes. Never claim a custom role/model ran without a genuine spawn-child-wait trace accepted by `validate-subagent-trace.js`.

### Template Update Protocol

For updates, follow `docs/TEMPLATE_RELEASES.md#canonical-agent-update-protocol`: classify source/downstream, pin one tag, verify manifest/diff/checks, never self-sync.

### Systemic Error Analysis

When an error, failed check, regression, or user correction appears, do not patch only the local symptom unless it is XS and isolated.

Before fixing, classify the failure:

- Local typo or one-off input issue: fix narrowly and verify.
- Broken contract between modules, docs, agents, hooks, routes, or SOTs: map the contract and fix the boundary.
- Repeated error or second failed attempt: stop local patching, diagnose root cause, and run the Change Strategy Gate before choosing the destination and transition.
- Architecture or workflow smell discovered during reading: run the Change Strategy Gate before the first patch; reroute once only when pipeline, risk, or approval authority changes.

Use the systemic debug shape:

```text
Observed failure:
Immediate symptom:
Likely broken link:
Root cause hypothesis:
Smallest systemic fix:
Regression guard:
```

Prefer a root-cause fix plus a regression guard over a local workaround. If the systemic fix changes scope, timeline, ownership, or quality bar, ask the user before applying it.

When the Change Strategy Gate fires, notify the user even if reversible internal
replacement can continue automatically. Record the decision in the active
orchestrator artifact; parent Codex uses `tasks/current.md`, while read-only work
may report it in the response. Optional machine-readable decisions use
`tasks/change-strategy.json` and must pass
`node scripts/validate-change-strategy.js tasks/change-strategy.json` before the
next state-changing patch.

### Thinking Tools Gate

For M+, HIGH-risk, ambiguous, template, architecture, design, product, or repeated-failure work, use explicit thinking tools before choosing the fix.

1. System map: name the real user outcome, app-specific KPI, active SOTs, contracts between files/agents/tools, bottleneck, and feedback loop.
2. TRIZ contradiction gate: when requirements fight, phrase the contradiction as "we need X without causing Y"; list existing resources; try separation by time, place, scope, mode, or user segment; prefer an ideal final result where the harmful tradeoff disappears instead of splitting the difference.
3. Change Strategy Gate: compare the current destination and transition with at least one real alternative; preserve user/data/public contracts, not implementation. Treat "simpler", "faster", and "more maintainable" as claims requiring a baseline and measured, observed, estimated, or unknown evidence.
4. Sun Tzu / stratagem terrain check: know the terrain, alternatives, competitors, constraints, center of gravity, and favorable ground; prefer winning without direct confrontation; use asymmetry and timing, not deception, dark patterns, or user-hostile manipulation.
5. Plan Reality Check: plan only after understanding result, user, business outcome, dependencies, critical path, parallel work, external risks, and real deadline reason.
6. Treat plans as forecasts, not promises: name the first useful iteration, the next verifiable checkpoint, and the first signal that the plan is drifting.
7. If the plan breaks, replan explicitly: old assumption, new reality, impact, options, recommendation, and what needs user approval.
8. No hidden drift: if time, budget, risk, or quality changes, tell the user before continuing silently.

Do not turn these tools into theater. If they do not change the decision, keep the note short. If they reveal a product, SOT, scope, deadline, or quality-bar conflict, ask the user with options.

### User Context
- User communicates in Russian (primary) and English (technical)
- Prefers direct communication, technical depth, no fluff
- Values: working code over perfect code, but hates regressions
- Common request patterns: "сделай X", "почини Y", "проверь Z"

### Task Formulation Examples

Use these examples to translate vague requests into an execution contract. They are adapted from the project SOT references: Spec Kit (`spec -> plan -> tasks`), Boris Tane (`plan as contract`), Ian Bull (`boundaries and change review`), HumanLayer/12-factor agents (`explicit context and control flow`), TRIZ contradiction framing, and the internal Ilyakhov planning note.

| User says | Agent should formulate | Expected behavior |
| --- | --- | --- |
| `сделай нормально` | "User wants the product flow to meet the production quality bar. Success means the primary user can complete [flow] with verified happy/error/empty states." | Ask only for missing product intent; otherwise inspect flow, plan, implement, verify. |
| `почини ошибку` | "User wants the observed failure removed and the broken link identified. Success means the symptom is gone and a regression guard covers the root cause." | Reproduce or document why blocked, map root cause, patch boundary, add/identify guard. |
| `обнови шаблон` | "User wants a verified tagged update without touching project-owned overlays." | Classify source/downstream, pinned preview/apply, verify manifest/diff/checks. |
| `улучши дизайн` | "User wants the current screen to serve one user job better and protect the app-specific KPI." | Run subtraction first, use tokens/components, verify rendered desktop/mobile states. |
| `проверь` | "User wants a findings-first review with severity, evidence, user/business impact, and smallest fix." | Do not summarize first; list concrete defects, gaps, residual risk. |
| `спланируй` | "User wants a decision-ready plan: first useful result, dependencies, options, risks, evidence, and replan trigger." | Give a progressive JPEG plan, not a task dump. |
| `когда будет готово?` | "User wants a reliable forecast. If the final date is unknown, success means the next verifiable checkpoint is concrete." | Do not invent certainty; give dependency, checkpoint, confidence, and what would move the date. |
| `план поехал` | "User wants control restored: old assumption, new reality, impact, options, and recommendation." | Replan before continuing; ask for approval when scope, date, cost, or quality changes. |
| `становится дольше/дороже` | "User wants hidden effort drift surfaced before more budget or attention is spent." | State time/risk/quality cost, options, and recommendation; do not report motion as value. |
| `требования конфликтуют` | "User wants the contradiction solved, not averaged: need X without causing Y." | Use TRIZ contradiction gate; propose separation/resource options and recommend the least harmful reversible move. |
| `мы опять чиним то же самое` | "User wants the repair loop stopped and the system path reconsidered without risking live contracts." | Fire Change Strategy Gate; classify posture/contracts, choose destination and transition separately, show evidence and approval boundary. |
| `проверь маркетинг` | "User wants a GTM/communication review tied to revenue or another app-specific KPI." | Check ICP, positioning, offer, funnel/buyer journey, channels, proof, measurement, and ethical risks before rewriting copy. |
| `примени Сунь-цзы/стратагемы` | "User wants competitive strategy, not ornamental quotes." | Map terrain, center of gravity, asymmetry, timing, and favorable ground; reject deception or dark patterns. |

## Shared Rules (Single Source of Truth)

Rules live in `.claude/library/`. **Read them before implementing.**

### Route-selected rules before code changes:

Prefer the shared rules returned by `scripts/codex-route-task.js`. If the router is unavailable or ambiguous, read this fallback set before code changes:

1. `.claude/library/process/research-first.md` — research before code
2. `.claude/library/process/self-verification.md` — doubt protocol, confidence declaration
3. `.claude/library/technical/code-style.md` — naming, immutability, types, functions-in-modules
4. `.claude/library/technical/architecture.md` — module boundaries, file size limits
5. `docs/SHARED_CONVENTIONS.md` — functions-in-modules, entry point rules
6. `.claude/library/product/production-product-standard.md` — final product quality bar
7. `.claude/library/process/product-goal-loop.md` — persistent product goal/current-step loop
8. `.claude/library/process/client-executor-contract.md` — client/executor accountability, anti-sycophancy, and evidence-before-done
9. `.claude/library/process/change-strategy-gate.md` — destination/transition decision, evidence, and approval boundary

### Read per task type:
- **Implementation**: also read `.claude/library/process/plan-first.md`
- **Review/Audit**: also read `.claude/library/meta/critical-thinking.md`
- **Writing content**: use `$codex-writing-workflow` and read `.claude/library/technical/writing.md`; add `$codex-technical-writing` when truth depends on code or runtime behavior
- **Work reports / closeout**: also read `.claude/library/technical/writing.md` and follow the client-facing report rules
- **Testing**: also read `.claude/library/technical/testing.md`
- **Design/UI**: also read `.claude/library/domain/domain-design-pipeline.md`

## Codex Skills

Codex repo-scoped skills live in `.agents/skills/`.

Use the skills named by the route output. Open only those `SKILL.md` files.

Project-specific Codex skills use `.agents/skills/project-*` and must be preserved by template sync.

## Codex Subagents

Project-scoped Codex subagents live in `.codex/agents/*.toml`.

Use subagents named by the route output and profiles in `scripts/codex-agent-policy.js`. `required` and `recommended` fan-out is proactive only for independent material value, with one automatic wave. Luna serves bounded `scout`, `log_analyst`, and `summarizer` work; Terra handles research/testing/isolated implementation; Sol handles judgment-heavy review. Parent Codex consolidates and edits. Never exceed `xhigh`.

Before fan-out, check for Spec Kit, litkit, Kiro, AgentOS, or project-local `project-*` workflow artifacts. If `spec.md`, `plan.md`, `tasks.md`, or equivalent task graphs exist, use them as the input contract. Treat `[P]` or equivalent metadata as the default signal for safe parallel work.

Routing details and copy-ready prompts: `docs/CODEX_FANOUT_PATTERNS.md`.

### AgentOS Boundary

AgentOS, when present, is the orchestrator. Codex must not create a competing task graph. Treat AgentOS Strategy/Tactic/Plan/Todo/Gate artifacts as the source of truth and use Codex routes as worker execution contracts.

If AgentOS is absent, the parent Codex thread is the orchestrator: it owns sequencing, consolidation, edits, verification, and release notes.

Template releases belong to this repository. Downstream projects and AgentOS workspaces consume released template versions through git tags and `scripts/sync-template.sh --from-git --ref <tag>`.

## Code Conventions (Critical Subset — Inline)

These rules are ALWAYS enforced. Full details in the library files above.

### Style
- No `any` type — use `unknown` + type guards
- No mutations — return new objects (const/final/let by default)
- Max **375 lines** per file — split if approaching
- Max **30 lines** per function — extract helpers
- Pure functions preferred (same input → same output)
- Guard clauses (early returns) over nested conditionals
- No magic numbers — named constants only
- No boolean parameters — use options objects

### Architecture
- **Functions-in-modules**: all business logic in importable modules, entry points only import and call
- Vertical slice architecture (organize by feature, not by layer)
- No circular dependencies
- Error handling at boundaries, not deep inside
- No hardcoded URLs, ports, credentials — use env vars

### Naming
- Functions: verb + noun (`getUser`, `calculateScore`)
- Booleans: is/has/can/should prefix (`isActive`, `hasPermission`)
- Constants: UPPER_SNAKE_CASE
- Files: kebab-case or snake_case (match language convention)

## Context Loading Protocol

At task start, read these project files to understand context:

1. `PROJECT_SPEC.md` — stack, dependencies, file structure
2. `tasks/current.md` — active work, handoff notes from previous session
3. `tasks/lessons.md` — past mistakes to avoid (read ALL entries)
4. `_reference/tool-registry.md` — existing utilities (SEARCH before creating new ones)

## DON'T
- Code files > 375 lines — split them
- No `any` — use `unknown` + type guards
- No mutations — return new objects
- No business logic in entry points — use functions-in-modules pattern
- No editing main/master directly
- No skipping tests before commit
- No committing secrets (.env, API keys)
- No presenting solutions without self-verification
- No new code without checking tool-registry first
- No hardcoded visual values (use tokens)
- No building screens without components (system→tokens→components→screens)
- No "you're right!" without logging WHY
- No surface-level analysis ("works"=HTTP 200 is NOT analysis)
- No writing files in non-UTF-8 encoding — ALL files must be UTF-8 without BOM (see docs/SHARED_CONVENTIONS.md)
- No mojibake, replacement characters, or mixed line endings in tracked text — run `node scripts/validate-text-policy.js`
- No raw `uname`, `/tmp`, or `mktemp` outside `scripts/lib/platform.sh` — use platform helpers so Windows is never treated like Linux

## Verification Before Completion

After implementing, before presenting results:

1. **Re-read the original request** (not your interpretation — the user's actual words)
2. **Match to success criteria** — does the solution achieve what was stated?
3. **State confidence**: HIGH / MEDIUM / LOW with reasoning
4. **State doubt**: what you're least sure about (NEVER "none")
5. If **LOW** confidence → present 2-3 options with trade-offs, don't implement

### Sunk Cost Test
> "If I had NOT already written this code, would I choose this exact approach?"
- YES → continue
- NO → run the Change Strategy Gate; replace or migrate only after protected contracts, evidence, rollback, and approval boundaries are explicit

## Design Work — HARD RULES (Figma, CSS, UI)
1. NEVER hardcode visual values. Use tokens/variables.
2. NEVER build from raw shapes. Use components.
3. Build order: System → Tokens → Components → Screens. NEVER skip to screens.
4. Every container must have layout mode (auto-layout / flexbox / grid).
5. 8 states: Default, Hover, Active, Focus, Disabled, Loading, Error, Empty.
6. Use `$codex-design-workflow` for design/UI work and `$codex-figma-workflow` for Figma MCP work.

## OpenAI Model Guidance

For current OpenAI API model selection, check official OpenAI docs and `docs/OPENAI_MODEL_GUIDANCE.md`.
As of the 2026-07-11 docs check, use GPT-5.6 Sol for demanding judgment, Terra for substantive support work, and Luna only for bounded discovery, log extraction, and summarization. Profiles are role-specific and capped at `xhigh`.
Do not hardcode model, effort, approval, or sandbox defaults in project `.codex/config.toml`; those stay in user or IDE config.

## Self-Improvement
After each correction: classify type (BUG/KNOWLEDGE_GAP/STYLE/DESIGN_DISAGREEMENT/MISUNDERSTANDING).
BUG or KNOWLEDGE_GAP → log to tasks/lessons.md with Track (BUG/KNOWLEDGE/PATTERN/PROCESS) + Severity (P0-P3).
When >50 entries → promote via `/weekly`.

## Token Economy
- Trust skills/memory over re-reading. Don't re-read files you read this session.
- Only read files you WILL use. Parallelize independent tool calls.
- Route outputs >20 lines to subagents. After 2 failed corrections → /clear.
- Task switching → HANDOFF.md (status + files + next steps), fresh session.

## Dual-Agent Coexistence
This project supports both Claude Code and OpenAI Codex.
- Shared rules: `.claude/library/` (single source of truth for both agents)
- Shared conventions: `docs/SHARED_CONVENTIONS.md`
- Claude-specific: `CLAUDE.md`, `.claude/settings.json`, `.claude/hooks/`
- Codex-specific: `AGENTS.md`, `.codex/config.toml`, `.codex/hooks.json`, `.agents/skills/`
- Sync check: `bash scripts/sync-agents.sh`

## Build & Test
<!-- Filled per-project -->
Not configured yet. Run project setup to populate.

## Work Report Style
Final reports about completed work must follow the client-facing report rules in `.claude/library/technical/writing.md`: lead with result, explain the effect in the reader's world, keep technical detail only when it changes a decision, and default to `Что было → Что стало → Что это даёт → Чего ожидать дальше`.

## Compaction
After compaction: re-read `tasks/current.md` and `AGENTS.md` to recover context.

## Template Version
4.9.0
