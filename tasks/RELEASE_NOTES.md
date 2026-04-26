# Agent OS 0.2.34

- Added a committed `v0.2.34` product UX/orchestration plan with 20 top-level improvements and 50 concrete implementation items.
- Added a unified Route Card in chat showing the next-message target, provider/model, mode/access, runtime state, history depth, scope, model source, and delegation counts.
- Moved common solo routing controls into the Route Card and hid duplicate header/route-strip controls to reduce cockpit noise.
- Added route warnings for offline providers, read-only execution mismatch, and Duo execution without a write-enabled lead.
- Added Duo route actions for ask-both, make-plan, and lead-exec from the same route surface.
- Added graph/code context attach actions so Graph Inspector can either attach context into the composer or ask the orchestrator immediately.
- Added backend provider metadata for Codex model source/count and backend Stop metadata for whether a provider PID was actually killed.
- Improved Stop UX to show `stopping` while provider cleanup is in progress and then settle from stream events.
- Hid bulk delegation approval behind a compact control while keeping per-delegation approve/reject visible.

# Agent OS 0.2.33

- Added a committed execution plan for the runtime/chat pass so release work is not only stored in the conversation.
- Fixed Codex chat cancellation at the process level: Codex CLI runs now register their child PID, and Stop can kill the tracked process tree instead of waiting for the CLI to return naturally.
- Added race-safe PID untracking so a late-finishing old provider process cannot remove the PID for a newer run in the same chat.
- Added an immediate cancelled `done` stream marker from Stop so the frontend can settle the run without waiting for provider cleanup.
- Added paginated chat history loading with `before/limit`, `total`, `loaded`, `has_more`, and `next_before` metadata.
- Added a `load older` control at the top of chat, preserving scroll position while older history is prepended.

# Agent OS 0.2.32

- Fixed chat auto-scroll during active thinking/streaming: the chat now stays where the user scrolls and only follows output while already near the bottom.
- Fixed Stop UX so visible partial output is preserved instead of disappearing back to the last user message while cancellation is settling.
- Expanded chat history loading from 50 to 200 JSONL entries and kept existing messages visible if a refresh fails.
- Made thinking/live-run output more visible with always-open thinking blocks, longer reasoning previews, and a persistent live bubble while a run is active.
- Added a compact solo route strip showing target, provider/model, mode, and access, with one-click context/review prompts instead of hidden routing.
- Reworked Graph view into a real SVG dependency map with selectable nodes, highlighted edges, group boxes, and inspector actions that route graph context/impact back into orchestrator chat.
- Guarded cancelled Codex runs from appending late stale responses after the user has already stopped the operation.

# Agent OS 0.2.31

- Fixed orchestrator/chat language drift: solo, plan mode, auto-continue, and Duo prompts now inject a shared response policy that keeps user-facing prose in the user's language, including Russian/Cyrillic conversations.
- Connected orchestrator prompts to the agent-template behavioral contract by injecting relevant template policy sections such as Philosophy, Work Report Style, and Don't rules.
- Stopped collapsing normal assistant answers after 800 characters; only raw diagnostics/command dumps keep the compact details rendering.
- Added regression tests for Russian response-policy detection and template policy section extraction.

# Agent OS 0.2.30

- Added a live run lifecycle for chat streaming: `run_started`, `run_progress`, and `run_done` events now describe provider, model, mode, access, phase, detail, and outcome.
- Added a live run HUD in chat so active work shows provider/model, `act/plan`, `read/write/full`, current phase, recent backend events, elapsed time, and terminal outcome instead of looking frozen.
- Made `poll_stream` return live activity/running/cancelled state so the frontend can reconcile status during long tool calls without waiting for the 15-second dashboard refresh.
- Added adaptive frontend refresh while work is active: activity updates every second, with project/feed/signal refreshes every few seconds during runs or active delegations.
- Improved Stop semantics in the UI and backend stream: cancelling emits a visible `cancelled` outcome and immediately updates the active run state.
- Improved delegation operability: status now prints full delegation IDs, status filters support pending/running, and cancel/retry/priority/timeout accept a unique delegation ID prefix.

# Agent OS 0.2.29

- Simplified solo chat controls to a KISS surface: one `act/plan` toggle, one `read/write/full` access selector, provider, model, and effort in the main chat header.
- Removed the nested solo `work area -> chat/compare/plan/execute` rail and the redundant composer route strip that made chat feel like a cockpit inside a cockpit.
- Made `Plan` mode a real backend contract: it forces read-only permissions and disables AgentOS PA command execution from that response.
- Wired `read/write/full` access into solo streaming so Codex/Claude receive the matching permission profile for the current message.
- Updated solo empty state and placeholder copy so the primary action is simply telling the selected agent what to do.

# Agent OS 0.2.28

- Replaced the fixed 3-turn auto-continue cap with a state-based AgentOS loop: continue while the agent produces actionable PA commands, stop when no more commands are emitted, when the user stops the chat, or when a repeat loop is detected.
- Raised the auto-run safety ceiling to 20 continuation turns as an emergency guardrail, not a normal workflow limit.
- Added chat cancellation state so `Stop` interrupts the AgentOS auto-run loop between command batches and follow-up agent turns, and the next chat run starts cleanly.
- Applied the same state-based command loop and repeat-loop guard to Duo execution leads.

# Agent OS 0.2.27

- Fixed desktop stream polling for project chats: the frontend now polls the correct per-project stream buffer instead of always reading `_orchestrator`.
- Extended chat streaming waits from 5 to 30 minutes and added a visible waiting heartbeat while agents or tools are still running.
- Added an AgentOS command auto-continue loop: command results are fed back into the selected agent for up to 3 continuation turns instead of requiring the user to type "continue".
- Applied the same auto-continue behavior to Duo execution leads, so execution does not stop after the first diagnostic/delegation command batch.
- Added live heartbeat refresh for Duo participant, round, and room actions during long operations.

# Agent OS 0.2.26

- Reworked the chat sidebar into a clearer working surface: context bar, active route, model/provider visibility, delegation counts, quick diagnostics, and a chat -> compare -> plan -> execute flow rail.
- Hid raw PA command/dump noise when a run card is available; standalone command-only replies now render as compact command batch cards with copy actions.
- Made run cards more actionable: filters for all/issues/outputs, capped scroll height, collapsed warning details, copy controls, and contextual hints for malformed delegation, permission, warning, and empty-result cases.
- Made Duo execution provider-neutral again: the primary handoff is `Lead executes`, with explicit lead choices in details instead of hardcoding Codex.
- Added chat-side provider refresh for Codex/ACP models and updated settings copy to explain ACP/cache/fallback model discovery including GPT-5.5.
- Fixed malformed-command warnings so `[DELEGATE_STATUS]`, `[DELEGATE_LOG]`, and `[DELEGATE_CANCEL]` no longer trigger the base `[DELEGATE:Project]...[/DELEGATE]` warning.

# Agent OS 0.2.25

- Reworked PA execution rendering into a compact `RunCard`: one run header, command rows, short summaries, and per-row details instead of a visible stdout wall.
- Added structured `pa_command` metadata to new PA command feedback events, so future run cards do not have to infer command/result links from assistant prose.
- Kept legacy chat compatibility through inference while making new streaming runs structurally matched by command.

# Agent OS 0.2.24

- Made the chat sidebar resizable from its left edge and widened assistant/tool bubbles to use the full panel width.
- Made PA execution traces compact: command results are collapsed by default, no-match outputs render as quiet row states, and batch commands are counted/matched to their results.
- Normalized common mojibake markers in PA trace output so old git/template results stop displaying broken check/arrow symbols.

# Agent OS 0.2.23

- Folded legacy PA `SYSTEM` messages from existing chat history into the previous assistant execution trace, so old conversations stop showing separate command-result bubbles.

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
