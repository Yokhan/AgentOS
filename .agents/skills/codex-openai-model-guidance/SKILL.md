---
name: codex-openai-model-guidance
description: "Use current official OpenAI docs for GPT-5.6 model selection, migration, Responses API, reasoning effort, tools, and prompt updates. Trigger on OpenAI model guidance."
---

# Codex OpenAI Model Guidance

OpenAI model guidance is volatile. Browse official OpenAI docs when the user asks for current recommendations.

## Local Reference

Read `docs/OPENAI_MODEL_GUIDANCE.md` for the latest verified snapshot in this template.

## Rules

- Do not hardcode model defaults in project `.codex/config.toml`.
- Prefer official OpenAI docs over bundled notes.
- For GPT-5.6 API work, start with Responses API and `reasoning.effort: "medium"` unless evals show another setting is better.
- This template never recommends or configures reasoning above `xhigh`.
- Cite official docs when answering the user.
