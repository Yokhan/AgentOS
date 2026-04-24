# Agent OS 0.2.8

- The right duo panel now exposes the active orchestrator and a direct `use Codex` / `use Codex as orchestrator` path without digging through runtime controls.
- Codex model choices now merge AgentOS fallbacks, the current saved model, Codex ACP capabilities, and the local Codex `models_cache.json`.
- New GPT-5-family models such as `gpt-5.5` are accepted instead of being reset to `auto`.
