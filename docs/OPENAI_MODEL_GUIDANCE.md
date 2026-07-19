# OpenAI Model Guidance

Verified against official OpenAI docs on 2026-07-11.

Sources:

- `https://openai.com/index/gpt-5-6/`
- `https://developers.openai.com/api/docs/guides/latest-model`
- `https://developers.openai.com/codex/subagents`
- `https://developers.openai.com/api/docs/guides/tools-multi-agent`

## Current Recommendation

Use the GPT-5.6 family for new complex reasoning, coding, tool-heavy, design,
research, and agent workflows.

- GPT-5.6 Sol is the quality-first tier for ambiguous work, architecture,
  security, design judgment, synthesis, and high-cost errors.
- GPT-5.6 Terra is the balanced tier for exploration, documentation research,
  test planning, isolated implementation, and parallel support work.
- GPT-5.6 Luna is the high-volume efficiency tier for bounded discovery, log
  extraction, and evidence condensation. It is not used for implementation,
  product judgment, architecture, security, or final verification without
  project-specific eval evidence.

For Codex parent sessions, do not hardcode the model in project config. Model
and reasoning effort remain user or IDE settings. The recommended parent
baseline is GPT-5.6 Sol with `medium` effort, raised only when task evidence
justifies it.

## Template Reasoning Ceiling

This template uses `xhigh` as the hard ceiling for parent recommendations and
subagent profiles. Higher settings are intentionally excluded from template
policy because automatic fan-out already increases compute and token use.

- `medium`: balanced default for exploration, docs, and test strategy.
- `high`: complex implementation, product review, design review, and general
  correctness review.
- `xhigh`: security and systems review where missed assumptions have a high
  cost.

Do not choose effort from role prestige. Choose it from ambiguity, dependency
depth, reversibility, evidence requirements, and the cost of a wrong answer.

## Codex Agent Profiles

The machine-readable source of truth is `scripts/codex-agent-policy.js`.
Runtime declarations under `.codex/agents/*.toml` must match it.

The parent model remains user-owned. Role-specific custom agents may pin a
model and effort because that is a specialist execution contract, not a
project-wide session default.

Template Luna roles are deliberately narrow: `scout`, `log_analyst`, and
`summarizer`, all at `low`. Terra handles research, test strategy, and isolated
implementation; Sol handles judgment-heavy review. Automatic fan-out is capped
at one wave and requires task evidence of parallel value.

Do not infer that a child used its pinned model from parent output. Runtime
verification requires a correlated trace accepted by
`node scripts/validate-subagent-trace.js`.

## GPT-5.6 Workflow Guidance

- Prefer the Responses API for reasoning, tool calling, multimodal, and
  multi-turn workflows.
- Start migrations at the current reasoning level, then evaluate the same level
  and one level lower on representative tasks.
- Use Programmatic Tool Calling for bounded filtering, joining, ranking,
  deduplication, aggregation, or validation where intermediate outputs can be
  reduced without fresh model judgment after every call.
- Use multi-agent work only when tasks divide into independent lanes. More
  agents are not evidence of better work.
- Measure final task success, evidence completeness, correction count, total
  tokens, latency, and cost. Fewer calls matter only when the user-visible
  result still meets the quality bar.

## Prompting Direction

GPT-5.6 benefits from shorter, outcome-focused prompts.

1. State outcome, constraints, permissions, evidence, success criteria, and
   output shape.
2. Remove repeated instructions and examples that no longer correct a measured
   failure.
3. Keep hot `AGENTS.md` guidance directional; put workflows in skills and
   stable references in docs.
4. Expose only task-relevant tools and keep tool descriptions precise.
5. Benchmark representative workflows instead of treating a release benchmark
   as a complete routing policy.

## Template Policy

Allowed:

- Documentation about recommended OpenAI models.
- Role-specific model and effort settings in `.codex/agents/*.toml` validated
  against `scripts/codex-agent-policy.js`.
- Skills and routes that automatically select useful independent subagents.
- User-level or IDE-level parent model selection.

Not allowed:

- A project-wide `model` or `model_reasoning_effort` in `.codex/config.toml`.
- Project defaults for approval policy or sandbox mode.
- Reasoning above the template's `xhigh` ceiling.
- Unconditional fan-out for XS tasks or overlapping write scopes.
- Stale model recommendations in template-owned instructions.
