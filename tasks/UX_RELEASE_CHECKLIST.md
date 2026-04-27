# AgentOS UX Release Checklist

Use before tagging a desktop release.

1. Route persistence: select a project, press Ctrl+R, verify the same project, chat route, mode, model, and access remain visible.
2. Duo toggle: switch solo -> duo -> solo from the header and chat header; center canvas must stay on dashboard/project/workbench.
3. Strategy load: open Strategy and verify no startup crash, no missing hook/import error, and generation controls still render.
4. Live run recovery: start a chat run, refresh during execution, verify live status shows phase, detail, elapsed time, stop, copy, and details.
5. Cancellation: stop a running chat and verify already streamed output remains visible until persisted history replaces it.
6. Context chips: attach project context and graph context, remove one chip, send, verify visible user message stays clean.
7. Project rail: use attention/active/stale/dirty/deleg/plan filters, select a project, return, and verify scroll position persists.
8. Keyboard navigation: focus project rail, use Up/Down, Enter, Escape; selected project and orchestrator route should update predictably.
9. Project detail: use "ask next safe step"; verify project context chip appears and composer draft is understandable.
10. Checks: run `npm.cmd run check:ui` and `npm.cmd test` before build/release.
