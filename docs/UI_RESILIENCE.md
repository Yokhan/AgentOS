# UI Resilience

## What Changed

Agent OS now has a runtime resilience layer for the desktop UI:

- UI long tasks are detected with `PerformanceObserver`.
- Event-loop stalls are detected by a watchdog.
- Window errors and unhandled promise rejections are captured.
- Diagnostics are stored locally and appended to `tasks/.ui-diagnostics.jsonl`.
- Safe mode can disable graph, execution map, orchestration map, timeline, and operation snapshot polling while keeping chat and core project data available.

## Why this was architecturally possible

The freeze class was possible because the UI had no strict owner for live data refresh.

- Global signals such as `executionMap`, `orchestrationMap`, `delegations`, and `operationSnapshot` are mutable from any module.
- Components could import heavy loaders and call them from render-adjacent effects.
- Startup, route refresh, chat state, dashboard polling, and execution-map UI could all request the same heavy backend state.
- Heavy refreshes were coalesced only after the request reached the API layer, not before a component decided to schedule work.
- There was no runtime backpressure signal, no safe mode, and no UI-side hang log, so the app could freeze without leaving useful local evidence.

## Guardrails

Only `app.js` owns automatic polling.

Components may request manual refresh from explicit user actions. They must not own repeated timers or background polling loops.

Heavy loaders must be:

- Coalesced in `api.js`.
- Safe-mode aware.
- Deferred until after the first shell render when used at startup.
- Covered by `scripts/check-ui-resilience.mjs` and `scripts/check-stream-performance.mjs`.

Startup must render the app shell before expensive live-state work.

Safe mode must keep these capabilities available:

- Chat history and message sending.
- Project list and project switching.
- Basic activity/feed/plan refresh.
- Settings and permissions.

Safe mode may disable these capabilities until the user turns it off:

- Graph view.
- Live execution map.
- Orchestration map.
- Execution timeline.
- Operation snapshot polling.
- Delegation stream polling.

## Operational Use

If the UI freezes or the startup error screen appears:

1. Enable safe mode from the startup error page or run `window.__AGENTOS_SET_SAFE_MODE__(true)` in DevTools.
2. Restart Agent OS.
3. Inspect `tasks/.ui-diagnostics.jsonl`.
4. Fix the source of long tasks or event-loop lag before disabling safe mode.

## Non-Negotiable Rule

If a feature needs live refresh, it must feed the centralized polling owner or expose a manual refresh button. It must not add another component-owned polling loop.
