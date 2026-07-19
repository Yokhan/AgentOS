---
name: codex-agent-router
description: "Route Codex work to repository skills, shared rules, plans, memory, and verification gates without Claude model-routing assumptions. Trigger when choosing workflow or skill coverage."
---

# Codex Agent Router

Codex does not use the Claude subagent/model routing table. Route by task type and risk inside the current agent.

## Mandatory First Step

For any file edit, M+ task, template work, release, design, security task, or ambiguous request, run:

```bash
node scripts/codex-route-task.js "<user request>" --summary --write-state
```

State the returned modes, exact/semantic matches, skills, pipeline, subagents, fan-out status/reason, risk, and orchestrator before editing. If the route reports `agentos`, treat AgentOS as the orchestrator and Codex as the worker.

The route's candidates are governed by `scripts/codex-agent-policy.js`. Automatically spawn `required` or genuinely useful `recommended` independent lanes without waiting for the user to request subagents. User opt-out wins; report why a candidate was skipped.

Routing is not keyword-only. `scripts/codex-route-task.js` combines exact patterns with semantic intent scoring from `scripts/lib/codex-route-intents.js`. When a task is misrouted, update the relevant intent groups and add a regression fixture instead of only adding one literal keyword.

Before edits, state a compact strategy for ambiguous, M+, HIGH risk, template, release, security, design, or cross-project work:

`Goal -> Constraints -> Approach -> Verification -> Risk/Doubt`

If the route includes `$codex-strategic-review`, use it before choosing the implementation path.

## Routing

- Implementation: `$codex-feature-workflow`, `$codex-pipeline-workflow`.
- Bugfix: `$codex-debug`.
- Review or audit: `$codex-audit`, `$codex-domain-software-review`, `$codex-domain-design-review`.
- Security: `$codex-security-audit`.
- UI/Figma: `$codex-design-workflow`, `$codex-figma-workflow`.
- Product goal/current-step continuity: `$codex-product-goal`.
- Literary, marketing/advertising, informational, and communication writing: `$codex-writing-workflow`; choose by reader purpose, not keywords alone.
- Marketing, GTM, positioning, funnel, campaigns, offer, ICP, and sales messaging: `$codex-writing-workflow`, `$codex-domain-communication-review`, `$codex-domain-business-review`, `$codex-product-goal`, `$codex-strategic-review`.
- Design system, Storybook, tokens, component contracts: `$codex-design-system-workflow`.
- Product UX flow/dead-end checks: `$codex-product-ux-audit`.
- Repeated downstream mistakes and template promotion: `$codex-cross-project-lessons`.
- Template changes: `$codex-template-sync`, `$codex-skill-maintenance`, `$codex-test-rules`.
- OpenAI API/model guidance: `$codex-openai-model-guidance`.
- Strategy, roadmap, release sequencing, ambiguous or HIGH-risk work: `$codex-strategic-review`, `$codex-decompose`.

For product, feature, design, game, API, or template implementation, route
progressive JPEG iteration planning to `$codex-progressive-jpeg-planner`; route implementation complaints to `$codex-product-goal`, `$codex-feature-workflow`,
and `$codex-strategic-review`: the expected behavior is an end-state skeleton
with 1% callable future capabilities, not legacy harness proof. Complaints about
old iterations, disabled scaffolds, stale placeholders, skipped tests, or
release-only exclusions also require the progressive layer replacement gate:
delete, replace, or time-box the superseded layer.

Repeated patching, compatibility shims that only preserve an old path,
architecture drift, stale-path tests, and repair-versus-rewrite decisions route
semantically to `$codex-change-strategy`. A normal first local defect should not
load the full gate.

During reading, invoke that overlay immediately when causal evidence reveals a
qualifying system mismatch. Reroute the original task at most once and only
when pipeline, risk, or approval authority changes. A discovery route with
`blockEdits: true` resumes its original pipeline after a valid, unblocked Change
Strategy decision; it must not create a nested pipeline or stay blocked.

If no skill fits, read the shared `.claude/library/` rules listed in `AGENTS.md` and state the chosen workflow before editing.
