# Agent OS 0.2.15

- Fixed PA command parsing after provider failures: Codex/OpenAI error output that echoes `[IDENTITY]` or prompt context is no longer scanned for executable AgentOS commands.
- Ignored command examples inside context blocks and fenced code, so placeholders like `[DELEGATE:Project]`, `[CRON_CREATE:name:schedule]`, and `[INCOME_RECORD:amount:category]` cannot create real delegations, cron entries, alerts, income records, or strategies.
- Added regression tests for provider-error echoes, identity blocks, fenced examples, and real command parsing.
- Updated the local Codex CLI to `0.125.0`; `gpt-5.5` now runs through the installed CLI instead of failing with the old-version 400 error.

# Agent OS 0.2.14

- Made the Duo handoff explicit: after a two-agent round the primary action is now `Make plan`, with a direct `Codex leads execution` path that switches a write-enabled Codex participant into the orchestrator role.
- Added `open in Duo execute` from the Plans view so a plan can be discussed, scoped, converted into tasks, and executed without hunting through panels.
- Renamed cockpit-style labels in the execution UI: provider batch buttons now say `run all Codex/Claude tasks`, and manual creation says `create task` instead of internal todo/workflow wording.
- Kept the underlying model/provider-neutral pipeline intact: Codex can lead execution when selected, while Claude/Opus and Codex child work still route through delegated task execution.

# Agent OS 0.2.13

- Fixed solo provider routing: project chats no longer force Claude when the user selects Codex or when the configured solo/orchestrator provider is Codex.
- Added a visible solo provider selector (`auto`, `claude`, `codex`) next to model/effort controls.
- Passed the selected solo provider through the Tauri chat command so frontend choice and backend execution cannot drift.
- Added regression tests for explicit Codex solo routing and auto routing from the configured orchestrator provider.

# Agent OS 0.2.12

- Added a read-only orchestration scope resolver so Duo knows whether the current context is global, project, strategy, plan, or task instead of guessing from the visible panel.
- Reworked the compact Duo card around that scope: breadcrumb path, scoped actions, and a single lead/mode disclosure replace duplicated primary controls.
- Mirrored the same scope path in the main Duo workspace so chat, project room, plans, and execution board stay aligned.
- Added a regression test that verifies a linked plan wins over the project fallback when resolving active scope.

# Agent OS 0.2.11

- Made the Duo flow provider-neutral: the right panel now asks who should lead, lists all room lead candidates, and uses `Execute with <current lead>` instead of hardcoding Codex as the execution path.
- Codex remains one-click selectable when present, but Claude or any write-enabled participant can be made lead from the same compact control.

# Agent OS 0.2.10

- Simplified the Duo right panel so it presents one readable flow instead of a cockpit of internal modes: primary review/execution actions, a compact route line, and collapsed advanced controls.
- Reduced duplicate Duo status chrome by removing the always-visible mode tab row, lower workspace notice, and noisy next-step button cluster.

# Agent OS 0.2.9

- The startup updater now restarts Agent OS after a downloaded update is installed, so the newly installed UI becomes visible immediately instead of leaving the old running process on screen.

# Agent OS 0.2.8

- The right duo panel now exposes the active orchestrator and a direct `use Codex` / `use Codex as orchestrator` path without digging through runtime controls.
- Codex model choices now merge AgentOS fallbacks, the current saved model, Codex ACP capabilities, and the local Codex `models_cache.json`.
- New GPT-5-family models such as `gpt-5.5` are accepted instead of being reset to `auto`.
