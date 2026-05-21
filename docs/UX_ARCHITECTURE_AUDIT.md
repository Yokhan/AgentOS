# AgentOS UX architecture audit

Date: 2026-05-21

## Immediate findings

1. Delegation state had no single readable surface. Active cards existed inside chat/workbench details, but there was no dedicated control center for approvals, running project agents, failed routes and recent completions.
2. Delegation API contracts diverged. The Tauri command returned non-terminal delegations, the HTTP endpoint returned only pending delegations, and the frontend kept only a small subset of delegation fields.
3. Chat carried too much operational state. Warnings, PA feedback, inbox summaries and delegation cards competed with the conversation, making it hard to answer the basic question: what is running, what needs me, what failed.
4. UI module boundaries are weak. `chat.js` owns chat, live map, route decisions, delegation cards and timeline rendering; `views.js` owns dashboard/workspace composition. This creates visual conflicts and makes targeted fixes risky.
5. Existing checks covered pieces of the UX, but not the product contract that delegations must be visible outside chat with full execution metadata.

## Fix shipped in this slice

1. Added a first-class workspace tab: `Делегации`.
2. Added a delegation control view with filters for needs-user, running, pending, failed, done and all.
3. Added delegation cards with task, status, provider/reviewer, priority, timeout, batch/plan/work/session links, gate/review/usage and live stream hints.
4. Added direct actions: approve, reject, cancel, retry, status and open project.
5. Unified backend delegation snapshots so UI can show active plus recent terminal delegations.
6. Updated frontend merging to preserve full delegation payload instead of dropping metadata.
7. Added `check-delegation-workspace-ui.mjs` to prevent regression.

## Remaining architectural debt

1. Split `chat.js` into smaller modules: chat transcript, execution map, delegation widgets, route decision widgets and composer.
2. Move delegation UI components out of `views.js` into a dedicated `src-ui/components/delegations.js` module after the workspace contract stabilizes.
3. Normalize all operational events into one event contract so chat, timeline, notification center and execution map do not each infer semantics differently.
4. Add visual/story smoke coverage for the delegation workspace with sample pending/running/failed/done payloads.
5. Reduce dashboard density by making the center workspace primary and keeping project navigation as a left rail, not a second dashboard.
