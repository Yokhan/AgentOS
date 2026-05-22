# AgentOS UX Architecture Audit

Date: 2026-05-22

## Immediate Findings

1. Delegation state had no single readable surface. Active cards existed inside chat/workbench details, but there was no dedicated control center for approvals, running project agents, failed routes and recent completions.
2. Delegation API contracts diverged. The Tauri command returned non-terminal delegations, the HTTP endpoint returned only pending delegations, and the frontend kept only a small subset of delegation fields.
3. Chat carried too much operational state. Warnings, PA feedback, inbox summaries and delegation cards competed with the conversation, making it hard to answer: what is running, what needs me, what failed.
4. UI module boundaries were weak. `chat.js` owns chat, live map, route decisions, delegation cards and timeline rendering; `views.js` owns dashboard/workspace composition.
5. Existing checks covered pieces of the UX, but not the product contract that delegations must be visible outside chat with full execution metadata.

## Shipped Fixes

1. Added a first-class workspace tab: `Делегации`.
2. Added a delegation control view with filters for needs-user, running, pending, failed, done and all.
3. Added delegation cards with task, status, provider/reviewer, priority, timeout, batch/plan/work/session links, gate/review/usage and live stream hints.
4. Added direct actions: approve, reject, cancel, retry, status and open project.
5. Unified backend delegation snapshots so UI can show active plus recent terminal delegations.
6. Updated frontend merging to preserve full delegation payload instead of dropping metadata.
7. Moved delegation workspace implementation out of `views.js` into `src-ui/components/delegations.js`.
8. Moved notification center implementation out of `views.js` into `src-ui/components/notifications.js`.
9. Moved route decision panels and route command actions out of `views.js` into `src-ui/components/routes.js`.
10. Added regression gates: `check-delegation-workspace-ui.mjs`, `check-notification-center-ui.mjs` and `check-ui-architecture-boundaries.mjs`.

## Current Boundaries

1. `views.js` is the workspace composer. It may route to workspace tabs, but should not own large feature implementations.
2. `src-ui/components/delegations.js` owns delegation UX: filters, cards, actions and live stream hints.
3. `src-ui/components/notifications.js` owns notification filters, grouped rows and clear/refresh actions.
4. `src-ui/components/routes.js` owns route decision UI and command actions for status, retry, health and approval decisions.
5. `chat.js` still owns too much: transcript, live map, embedded route widgets and composer. This is the next split target.
6. `api.js` is still a broad frontend service layer. It should eventually be split by domain: chat, delegation, execution map, strategy/plans, provider settings.

## Remaining Debt

1. Split `chat.js` into smaller modules: chat transcript, execution map, route decision widgets and composer.
2. Normalize all operational events into one event contract so chat, timeline, notification center and execution map do not infer semantics differently.
3. Add visual/story smoke coverage for delegation, route decision, notification and execution map workspaces with sample pending/running/failed/done payloads.
4. Reduce dashboard density by making the center workspace primary and keeping project navigation as a left rail, not a second dashboard.
