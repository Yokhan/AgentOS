# Top Agent Engineering Works

Checked: 2026-05-26

This is the priority reading set for maintaining this template. It stores source
cards and local conclusions, not full third-party texts. Canonical URLs live in
`sources.json`; browse them again when changing behavior-sensitive mechanics.

## TW-01 Local AI Agent Spec v3

source_id: `local-ai-agent-spec-v3`

Use as the user-provided baseline for this template. Main pattern: three context
tiers, agent-ready architecture, hooks for deterministic enforcement, skills and
agents for specialist knowledge, and drift checks for regression control.

Apply here: keep `AGENTS.md`/`CLAUDE.md` as hot memory, move procedure into
skills, keep long sources in `_reference/agent-sot`, and enforce with scripts.

## TW-02 Codified Context

source_id: `arxiv-codified-context`

Best evidence for a multi-tier context system in a large codebase. It supports
the hot-memory constitution, specialist agents, and cold-memory documents model.

Apply here: template changes must preserve those three layers and avoid growing
one monolithic instruction file.

## TW-03 Agentic Context Engineering

source_id: `openreview-agentic-context-engineering`

Important for self-improving agents and for the risks of context collapse. It
also explains why over-compressing specialized knowledge into generic prompts is
dangerous.

Apply here: source cards can be concise, but skills and references must preserve
task-specific details when they matter.

## TW-04 Building Effective Agents

source_id: `anthropic-building-effective-agents`

Core production pattern catalog: workflows, routing, parallelization,
orchestrator-workers, evaluator-optimizer, and autonomous agents.

Apply here: prefer deterministic workflows and routing first; use subagents only
when task decomposition or parallel review earns the added complexity.

## TW-05 Effective Context Engineering

source_id: `anthropic-effective-context-engineering`

Frames context as finite infrastructure. Strongly supports compaction,
structured notes, multi-agent isolation, and careful retrieval.

Apply here: do not bulk-load all docs; route to precise skills and references.

## TW-06 Multi-Agent Research System

source_id: `anthropic-multi-agent-research-system`

Good model for lead-agent plus parallel specialist workers. Also reinforces
source-quality heuristics and stop conditions.

Apply here: parent Codex/Claude consolidates; workers explore, review, or test.

## TW-07 Context Engineering For Coding Agents

source_id: `martinfowler-context-engineering-coding-agents`

Separates kinds of context and explains why coding-agent UX is increasingly
defined by context enrichment.

Apply here: keep instructions, guidance, context interfaces, references, and
dynamic tool-fed context in different places.

## TW-08 Harness Engineering

source_id: `martinfowler-harness-engineering`

Agent reliability comes from a harness: feedforward guidance plus feedback
sensors. Repeated failures should improve the harness.

Apply here: every repeated agent failure becomes a rule, skill, hook, validator,
or scaffold, not another vague reminder.

## TW-09 Thoughtworks Technology Radar: Context Engineering

source_id: `thoughtworks-context-engineering-radar`

Useful industry signal that context engineering has become a foundational
architecture practice, not prompt polish.

Apply here: context should be dynamic, layered, and progressively disclosed.

## TW-10 GitHub Spec Kit

source_id: `github-spec-kit`

Best practical model for spec-driven agent workflows: Spec -> Plan -> Tasks ->
Implement with quality gates and agent portability.

Apply here: use spec-like artifacts when intent, constraints, and task ordering
must survive multiple sessions or agents.

## TW-11 Spec Kit Agents

source_id: `arxiv-spec-kit-agents`

Adds repository-grounding and validation hooks to spec-driven development.

Apply here: routes must discover existing Spec Kit/litkit/AgentOS artifacts and
treat them as the task contract.

## TW-12 Beads

source_id: `steve-yegge-beads`

Structured, git-backed task memory is better than long, decaying markdown plans
for multi-session agent work.

Apply here: keep `tasks/queue.md`/AgentOS/litkit integration compatible with a
machine-readable dependency graph model.

## TW-13 Annotated Plan Workflow

source_id: `boris-tane-claude-code-workflow`

The strongest human-in-the-loop planning pattern: research artifact, plan
artifact, human annotations, then execution.

Apply here: for risky work, produce a durable plan and let the user correct the
plan before edits when intent is not already explicit.

## TW-14 Sinks, Not Pipes

source_id: `ian-bull-sinks-not-pipes`

Agent-ready architecture needs discoverable boundaries, deep modules, and
contained side effects.

Apply here: preserve functions-in-modules, vertical slices, public entry points,
and blast-radius checks.

## TW-15 Working Memory Cliff

source_id: `ian-bull-working-memory-cliff`

Agents fail when asked to track too many independent items. The response is
chunking, external state, tools, and decomposition.

Apply here: route M+ work through subagents/checklists and keep hot instructions
short.

## TW-16 Planning Is The Bottleneck

source_id: `ian-bull-planning-bottleneck`

When code is cheap, judgment and sequencing become scarce.

Apply here: template should optimize for better decisions, explicit success
criteria, and change review, not raw code generation volume.

## TW-17 Change Reviews

source_id: `ian-bull-change-reviews`

Review intent, impact, risk, reversibility, and shared understanding rather than
style trivia.

Apply here: audits and final reviews must lead with behavioral risk and missing
verification, not formatting.

## TW-18 12 Factor Agents

source_id: `humanlayer-12-factor-agents`

Production agents are mostly software: explicit prompts, explicit context,
owned control flow, pause/resume, human contact, and small focused agents.

Apply here: keep route scripts, hooks, task state, and verification as software,
not only prose.

## TW-19 Scaffolding For Monorepos

source_id: `vuong-ngo-scaffolding-monorepo`

Scaffolding prevents convention drift better than repeating instructions in a
long prompt.

Apply here: add generators/templates for repeated project patterns before adding
more rules.

## TW-20 Boris Cherny Claude Code Customization

source_id: `boris-cherny-customization-tips`

High-throughput agent use depends on customization: parallel sessions, custom
agents, skills/MCP/tools, hooks, permissions, and status feedback.

Apply here: keep Codex/Claude customization surfaces explicit and testable while
leaving model/effort/sandbox defaults to user or IDE config.
