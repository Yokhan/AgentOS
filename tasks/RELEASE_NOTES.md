# Agent OS 0.2.22

- Grouped PA command feedback into the assistant chat turn instead of scattering execution output as separate `SYSTEM` messages.
- Rendered PA execution as collapsible command trace cards with command status, warnings, and long outputs collapsed.
- Hid raw standalone PA command lines such as `[TEMPLATE_AUDIT]` from assistant prose when the executed trace is available.

# Agent OS 0.2.21

- Added explicit `pa status` chat entries before and after each PA command, so long diagnostics show exactly which command is running instead of looking like a frozen assistant message.
- Added readable command labels for PA execution feedback, including delegation, git/template, dashboard, health, memory, cron, and graph commands.

# Agent OS 0.2.20

- Made chat the visible execution journal for PA commands: command results and warnings now stream live and persist as system messages in chat history.
- Fixed solo orchestrator stream rendering for PA results, so responses like `[DASHBOARD_FULL]` and `[TEMPLATE_AUDIT]` no longer look like inert text after execution.
- Limited Codex solo PA command execution to the orchestrator chat instead of project chats.
- Fixed parsing of multiple `[DELEGATE_STATUS:...]` commands in one agent response and added regression coverage for the diagnostic batch shown in chat.

# Agent OS 0.2.19

- Corrected the execution-lead prompt to use the real failed-delegation diagnostic command: `[DELEGATE_STATUS:?failed]`.
- Hardened Duo Execute so the selected lead is promoted through the orchestrator path if their room state does not currently have write enabled.

# Agent OS 0.2.18

- Fixed Duo Execute message routing: the composer now sends execution prompts to the selected room lead/orchestrator instead of falling back to solo chat.
- `ask both` now stays analysis-only for two-agent review, while `lead` switches to execution mode and runs the selected participant with PA command execution enabled.
- Clarified the composer route and placeholder text so the UI shows whether input will review with both agents or execute with the lead.
- Strengthened execution-lead prompting: PA commands must be emitted outside fenced code blocks, and common diagnostics/delegation commands are listed explicitly.

# Agent OS 0.2.17

- Fixed Codex write execution: Codex CLI now receives an explicit sandbox derived from the same AgentOS permission profile as Claude (`read-only`, `workspace-write`, or `danger-full-access`).
- Solo Codex chat and Duo Codex execution now pass the selected permission profile into the provider runner instead of dropping it.
- `Codex leads execution` now promotes Codex to orchestrator and grants write access in one action; lead buttons show when they will grant write.
- Added regression coverage for Codex sandbox mapping from restrictive/balanced/permissive profiles.

# Agent OS 0.2.16

- Made Codex runtime selection explicit: Settings now shows configured transport, effective route, CLI status, ACP status, and one-click `use CLI` / `use ACP` switches.
- Set the local Codex route to CLI for `gpt-5.5`; ACP is no longer allowed to look `ready` unless it can create an actual chat session.
- Compact provider failures before they reach chat: model/runtime/auth errors now show an actionable fix instead of raw stderr or echoed prompt context.
- Hardened Codex ACP handling against stdout log noise and made the optional ACP smoke test skip incompatible adapters instead of failing the whole suite.
- Fixed startup logging to report the package version instead of the stale `v0.2.0` string.

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
