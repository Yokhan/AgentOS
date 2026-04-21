# Agent OS — Desktop Command Center

## What This Is
Tauri desktop app for orchestrating Claude Code agents across multiple projects.
Rust backend (Axum API + Tauri commands) + Preact frontend (inline SPA + extracted CSS).

## Stack
- **Backend**: Rust, Tauri 2, Axum 0.8, Tokio, Serde
- **Frontend**: Preact (vendor bundle), vanilla CSS (design tokens), inline `<script type="module">`
- **Subprocess**: `claude -p` with `--stream-json` for real-time chat, `--continue` for session persistence
- **Config**: `n8n/config.json` (documents_dir, orchestrator, models, permissions)

## Map
- `src-tauri/src/` — Rust backend
  - `lib.rs` — app entry, tray, project root detection
  - `state.rs` — AppState, caches, delegations, per-dir locks, inbox
  - `api_server.rs` — HTTP API on :3333 (auth via bearer token)
  - `scanner.rs` — project scanning (parallel git calls)
  - `logger.rs` — file logger with rotation (5MB)
  - `commands/chat.rs` — sync chat (send_chat), chat history, export
  - `commands/chat_stream.rs` — streaming chat (stream_chat, poll_stream, stop_chat)
  - `commands/chat_parse.rs` — PA prompt builder (identity + projects + categories + delegations + strategies + plans + memory + history)
  - `commands/pa_commands.rs` — unified PA command parser + validator + executor (DELEGATE, DEPLOY, PLAN, QUEUE, NOTIFY, REMEMBER, STRATEGY)
  - `commands/category.rs` — category management: load categories.json, PA context, delegation enrichment
  - `commands/status.rs` — type-safe status enums (DelegationStatus, StrategyStatus, StepStatus, PlanStatus, PlanStepStatus)
  - `commands/claude_runner.rs` — subprocess management (run_claude, find_claude, safe_truncate, atomic_write, permission paths)
  - `commands/process_manager.rs` — PID tracking, activity tracking, process killing
  - `commands/delegation.rs` — delegation lifecycle: queue → approve → L1/L2/L3 escalation → result → inbox → strategy/plan step update
  - `commands/delegation_stream.rs` — real-time delegation streaming (stream buffer + poll), stage events
  - `commands/delegation_analytics.rs` — delegation logging and analytics queries
  - `commands/auto_approve.rs` — config-driven auto-approve rules, background loop (30s), scheduled delegation execution
  - `commands/usage.rs` — token/cost tracking per delegation, usage summary endpoint
  - `commands/inbox.rs` — agent feedback inbox, batch processing to PA
  - `commands/plans.rs` — execution plans CRUD, PA context builder, plan↔delegation linking
  - `commands/ops.rs` — deploy template, health check, create project, queue, telegram, attachments
  - `commands/config.rs` — permissions, settings, model config, audit trail
  - `commands/strategy.rs` — goals, strategies, step execution via delegation queue (not direct claude)
  - `commands/strategy_models.rs` — Strategy/Plan/Step structs, context builders, delegation↔strategy linking
  - `commands/pa_commands_deleg.rs` — 12 delegation extended command parsers (BATCH, CHAIN, RETRY, CANCEL, etc.)
  - `commands/pa_commands_ops.rs` — 30 ops command parsers (Deploy, Git, Memory, Cron, Comms, Financial)
  - `commands/delegation_ext.rs` — execution for 12 extended delegation commands
  - `commands/delegation_models.rs` — DelegationPriority enum, DelegationTemplate struct
  - `commands/deploy.rs` — SSH operations (SCP deploy, verify, rollback, server exec/status, nginx, SSL, DNS)
  - `commands/git_ops.rs` — cross-project git (bulk push/pull, status, stale branches, search, audits)
  - `commands/memory_ext.rs` — PA memory CRUD (list, search, delete with archiving)
  - `commands/cron.rs` — recurring task scheduling (create, list, edit, delete)
  - `commands/comms.rs` — reporting (daily report, dashboard, activity digest, alerts, partner updates)
  - `commands/financial.rs` — income tracking and financial dashboard
  - `commands/graph.rs` — Graph View commands: overview, project, verify, diff, subgraph, timeline, export mermaid
  - `commands/graph_models.rs` — GraphNode/Edge types, Tarjan SCC, layered layout, coupling metrics
  - `commands/graph_scan.rs` — graph builders: overview (projects+segments), file-level (regex imports JS/TS/Rust/Python/Godot), operations overlay, agent protocol context
  - `commands/chat_stream_poll.rs` — stream polling: poll_stream, stop_chat, is_chat_running
  - `commands/delegation_cmds.rs` — delegation Tauri commands: approve, reject, schedule, cancel
  - `commands/feed.rs` — activity feed, digest, project plan analysis
  - `commands/agents.rs` — agent/segment listing with caching
  - `commands/proxy.rs` — n8n webhook proxy (validated paths)
  - `commands/gate.rs` — Gate Pipeline: post-delegation verify (script + diff + cost), GateResult struct
  - `commands/signals.rs` — Signal System: emit/ack/count/context, SignalSource/Severity enums, incident detection
  - `commands/sensors.rs` — Sensor Framework: StrategyNext, IncidentPause, StaleProcess, CostGuard, VerifyTodos
- `src-ui/` — frontend
  - `index.html` — SPA shell + inline JS (~1150 lines, CSS extracted)
  - `styles/main.css` — layout, tokens, components (137 lines)
  - `styles/chat.css` — chat, messages, delegations, inbox (287 lines)
  - `utils.js` — helpers (md, ft, esc, beep) — 50 lines
  - `office.js` — pixel office visualization (not yet connected)
- `n8n/` — configs
  - `config.json` — main config (documents_dir, orchestrator, models, effort, permissions)
  - `dashboard/segments.json` — project grouping
  - `dashboard/categories.json` — category metadata (shared_resources, delegation_strategy, description)
  - `dashboard/permissions/` — Claude permission profiles (restrictive/balanced/permissive)
- `tasks/` — runtime data
  - `chats/` — per-project chat history (JSONL)
  - `plans/` — execution plans (JSON)
  - `.delegations.json` — delegation state
  - `.strategies.json` — strategy state
  - `.chat-history.jsonl` — activity feed
  - `.usage-log.jsonl` — token/cost tracking per delegation
  - `.delegation-log.jsonl` — delegation analytics log
  - `pa-memory.jsonl` — PA persistent notes
  - `.signals.jsonl` — signal system events (gate results, incidents, timeouts)
  - `queue.md` — todo queue (PA-visible via [QUEUE] context)
  - `PIPELINE.md` — full Strategy→Tactic→Plan→Todo pipeline spec
  - `agent-os.log` — application log (rotated at 5MB)

## Orchestrator Pipeline
```
Strategy → Tactic → Plan → Todo → Delegation → Gate → Signal → PA feedback loop

User message → build_full_pa_prompt (identity + projects + categories + delegations + strategies + gates + signals + queue + memory + history)
→ stream_chat (claude --continue -p --stream-json)
→ PA responds: text + structured commands ([DELEGATE:...], [PLAN:...], [STRATEGY:...], etc.)
→ parse_pa_commands → validate → execute_pa_command (unified pipeline)
→ [PLAN:...] creates Strategy with Tactic→Plan→Todo hierarchy
→ User approves steps → queue_delegation_internal → Delegation{pending}
→ User/auto-approve → acquire_dir_lock → L1(balanced)→L2(permissive)→L3(PA decides)
→ Safety rails: heartbeat watchdog (120s), token budget (150K), context rotation (3 fails → fresh)
→ Gate Pipeline: verify script (exit code) + git diff stats + cost check → GateResult{pass/warn/fail}
→ Signal System: gate results → emit signals → PA sees [SIGNALS] in context
→ Incident: 3+ critical/10min → auto-approve paused for project
→ Auto-trigger: critical signals → PA auto-invoked to fix
→ Sensor framework (30s): StrategyNext, IncidentPause, StaleProcess, CostGuard, VerifyTodos
→ VerifyTodos: Todo.verify condition checked by script → auto-mark done without LLM
→ Strategy auto-complete: all todos done → plan done → tactic done → strategy achieved
```

## Config Fields
```json
{
  "documents_dir": "~/Documents",
  "orchestrator_project": "PersonalAssistant",
  "orchestrator_model": "opus",
  "orchestrator_effort": "max",
  "delegation_model": "sonnet",
  "delegation_effort": "high",
  "project_permissions": {"Project": "balanced"}
}
```

## Concurrency Model
- **Per-directory lock**: `state.dir_busy` prevents two claude processes in same project
- **Background thread**: stream_chat spawns reader loop in std::thread, returns immediately
- **spawn_blocking**: delegation, strategy, inbox processing run in tokio blocking pool
- **PID tracking**: running_pids tracks claude processes, kill_existing prevents zombies

## Build & Test
```
npm install                    # install @tauri-apps/cli
npm run tauri dev              # dev mode with hot reload
npm run tauri build            # production build
cd src-tauri && cargo check    # verify Rust compilation
```

## Self-Modification Safety
1. **UI changes (index.html)** — safe, hot-reload on F5
2. **Rust changes** — require `cargo check` before commit
3. **Always `cargo check`** for any .rs file change
4. Watchdog restarts app on crash — but fix root cause

## DON'T
- Code files > 375 lines — split them
- No hardcoded visual values (use CSS tokens)
- No hardcoded font sizes (use --fs-* tokens)
- No `let _ = ...` on critical file I/O — at least log_warn
- No byte-slicing strings — use safe_truncate() or .chars().take(N)
- No hardcoded status strings — use enums from status.rs (StrategyStatus, StepStatus, PlanStatus, etc.)
- No two claude processes in same directory — use acquire_dir_lock
- No editing main/master directly
- No committing secrets (.env, API keys)
- No surface-level analysis ("works"=HTTP 200 is NOT analysis)
