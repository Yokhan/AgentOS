# Agent OS UX Polish Plan 50+20

Date: 2026-04-27

## Product Goal

Agent OS should increase the user's ability to manage many AI agents in parallel with less manual control.

This pass is not about adding new orchestration capability. It is a product polish pass for the existing system:

- make the current route obvious;
- make chat the primary control surface;
- make project navigation stable and predictable;
- make Duo feel like a mode of the same chat, not a second app;
- make buttons communicate what will happen before the user clicks;
- make long operations visibly alive;
- remove cockpit/debug noise from the daily path.

## Audit Findings

1. The app still exposes multiple competing surfaces for the same concept: header solo/duo toggle, route card controls, context drawer, legacy Duo view, embedded Duo panel, plan `open in Duo execute`, and old header `DUO`.
2. The legacy Duo screen still exists as a deprecated compatibility view, but the product shell still lets the user reach it, creating a mismatch with the new chat-first mental model.
3. The route card is too dense for a daily control surface: it mixes destination, model, permission, history, scope, delegation counts, warnings, provider state, and action buttons in one block.
4. Many buttons are visually equivalent even when they have very different risk levels: read-only prompt insertion, write execution, delegation approval, route queueing, and external app launch.
5. Several buttons only show a toast, but do not visibly change local UI state while the action is pending, so the user cannot tell whether the click worked.
6. The chat header still contains hidden duplicated controls through CSS while the route card contains the visible controls, which makes the DOM and UX model harder to reason about.
7. The chat context drawer hides important power tools, but its summary does not explain when opening it is useful.
8. The dynamic run feedback is split between `RunningBanner`, `LiveRunHud`, `TranscriptStatusBar`, `StreamBubble`, execution timeline, activities, and delegations, so the user sees fragments instead of one coherent live state.
9. The live state can degrade into a ticking timer when backend activity exists but rich stream events are absent or were lost across reload.
10. Current project persistence was added, but the UI still needs a stronger route identity model: selected project, chat project, active run project, Duo session project, and scope can still diverge.
11. Project navigation moved left, but the current rail is still a list, not a control instrument: it needs clearer grouping, active route marker, health badges, and keyboard behavior.
12. Strategy/Plans/Graph/Project detail are separate canvases, but chat does not consistently show which canvas context is attached to the next message.
13. Some source files contain mojibake in user-facing strings. This is a UX defect and a release-risk defect because prompts and labels can silently degrade.
14. Existing UI smoke checks caught startup crashes late, but they still do not validate button wiring, route state transitions, or visible loading states.
15. The plan/strategy hierarchy exists, but the visible UX does not consistently answer: which goal, which plan, which project, which agent, which next step?
16. Duo collaboration and Duo execution are both implemented, but their daily user story is not compressed enough: ask both, compare, choose lead, execute, review should be one visible flow.
17. Context and graph actions are powerful, but they are exposed as raw prompt insertion instead of a clear "attach context to next message" state.
18. External and risky actions lack confirmation/undo semantics: sync template, health check, approve all, queue route, execute next step.
19. Empty states exist, but they are generic; they should teach the next useful action for the current context.
20. Release checks validate syntax and some rendering, but not the top UX regressions that keep recurring: startup route reset, legacy Duo mismatch, dynamic output loss, and non-obvious controls.

## 20 Top-Level Improvements

1. Single route identity model.
   Define one frontend `activeRoute` concept that derives project, scope, chat key, Duo session, active run, and canvas context.

2. Chat-first command surface.
   Treat the chat composer as the only place where user intent enters the system; buttons should either fill the composer, change route/access, or execute a clearly labeled action.

3. Left project rail as navigation, not dashboard content.
   Keep projects in a persistent left rail and make the center canvas focus on state, plan, project detail, strategy, graph, or execution.

4. Route card simplification.
   Split daily route controls from debug metadata. Daily view shows target, mode, access, provider/model, and one primary next action.

5. Risk-based button language.
   Label actions by effect: "insert prompt", "refresh", "run read-only check", "execute", "approve", "open external app".

6. Visible pending state for every action.
   Any button that starts work must show pending, success, or failure in place, not only as a toast.

7. Unified live run strip.
   Merge `RunningBanner`, `LiveRunHud`, transcript live state, and backend activity into one live strip with phase, last event, elapsed, and stop/retry.

8. Durable dynamic output.
   Preserve partial output across cancellation, reload, delayed history write, and provider errors.

9. Duo as a chat mode, not a separate screen.
   Remove or fully redirect the legacy Duo screen. The header `DUO` must open the same chat Duo mode, not a second product surface.

10. Duo flow compression.
   Make the visible flow: ask both, compare result, choose lead, execute, review. Hide room internals unless expanded.

11. Project-agent route visualization.
   Show project-agent lanes as concise cards tied to the selected project and plan, not as a dense debug map.

12. Plan/strategy context visibility.
   In chat, show whether the next message is attached to global, project, strategy, plan, task, graph, or review context.

13. Context attachment state.
   Replace raw `[GRAPH_CONTEXT]` mental model with a visible "context attached" chip that can be removed before sending.

14. Encoding and copy cleanup.
   Eliminate mojibake in source and rendered UI; enforce UTF-8 and add a check for common mojibake markers.

15. Action taxonomy.
   Standardize actions into read, discuss, plan, execute, approve, inspect, external. Use consistent styles and keyboard behavior.

16. Better empty/error states.
   Empty states should explain why nothing is visible and offer one safe next action.

17. Better reload/recovery behavior.
   `Ctrl+R`, startup, and hot reload should preserve route, chat, active run, scroll intent, and context attachment.

18. Stronger UX regression checks.
   Add static and smoke checks for route persistence, missing imports, mojibake, duplicate legacy surfaces, and critical button wiring.

19. Accessibility and keyboard pass.
   Buttons need titles, focus states, disabled reasons, and keyboard-accessible navigation between rail, chat, and canvas.

20. Visual hierarchy pass.
   Reduce border noise, duplicated metadata, and equal-weight controls. Make primary intent and current state visually dominant.

## 50 Concrete Polish Tasks

1. Create a frontend `routeState` helper that normalizes `_orchestrator` vs project route, scope label, chat key, and display name.
2. Replace direct scattered `currentProject.value || "_orchestrator"` route formatting in chat/view code with the route helper.
3. Add a route mismatch warning if chat key, active run project, Duo session project, and selected project disagree.
4. Persist last selected project, active canvas, chat mode, and Duo tab under one namespaced localStorage object.
5. Make `Ctrl+R` run an in-app refresh by default and expose browser/hard reload only through an explicit debug shortcut.
6. Convert header `DUO` navigation into a direct toggle for chat Duo mode.
7. Replace `DualAgentsView` content with a redirect/empty compatibility page that sends the user back to chat Duo mode.
8. Remove or hide legacy Duo header route if the unified chat Duo mode is available.
9. Rename Duo states in UI to user terms: "solo", "compare", "execute", "review result".
10. Add a single Duo flow card that shows current step: compare, decide lead, execute, review.
11. Move advanced Duo controls into one disclosure named "advanced room controls".
12. Hide leases, batches, raw work item IDs, and participant internals by default.
13. Make "ask both" insert or send through one predictable path depending on the current mode.
14. Make "make plan" always show where the plan will be saved before execution.
15. Make "lead executes" show selected lead, write access, and target project before running.
16. Split route card into `RouteSummary` and `RouteDetails`.
17. Keep `RouteSummary` always visible and cap it to one compact row plus warning row.
18. Move model count, model source, history counts, and delegation counts into `RouteDetails`.
19. Add inline pending state to provider/model/access selects after change.
20. Add disabled reason text for execution buttons when write access or lead is missing.
21. Replace generic "project context" button with "attach project context".
22. Replace generic "graph context" button with "attach graph context".
23. Show attached context chips above composer: project, graph, plan, strategy, review.
24. Add remove buttons for context chips before send.
25. On send, include attached context and then clear only the chips that were consumed.
26. Convert raw prompt-insertion helper buttons into chips plus editable composer text.
27. Merge `RunningBanner` and `LiveRunHud` into one `LiveStatusStrip`.
28. Feed backend `activities` into the live strip even when no stream events are present.
29. Show last tool/command/action in the live strip, not only elapsed time.
30. Add stop, copy last output, and open details buttons to the live strip.
31. Keep `StreamBubble` visible after cancellation until a persisted assistant message replaces it.
32. Add a visible "recovered output" state that is not styled like an error.
33. Add action-local statuses for `sync template`, `health check`, `open in Zed`, `approve`, `reject`, `queue`, and `execute next step`.
34. Add confirmations for destructive or bulk actions: approve all, template sync, queue parallel batch.
35. Add an undo/cancel affordance for queued but not yet running work where backend supports it.
36. Make project rail selection visually stronger than hover.
37. Add project rail badges for active run, blockers, dirty count, stale, and pending delegation.
38. Add rail filters for attention, active, stale, dirty, has delegation, has plan.
39. Preserve rail scroll position when selecting a project.
40. Add keyboard navigation for project rail: up/down, enter, escape to orchestrator.
41. Make center workbench hero use the selected route and plan context instead of only global aggregate.
42. Convert project detail inline styles into classes so layout can be tuned consistently.
43. Reduce project detail panels to a default summary plus expandable modules/issues/context.
44. Add a single "next safe action" area in project detail tied to chat composer.
45. Fix all mojibake strings in `src-ui/chat.js`, `src-ui/pages.js`, and existing visible docs used in-app.
46. Add `scripts/check-mojibake.mjs` for common markers like `Р`, `В·`, `в†`, `вЂ`, `рџ` in user-facing UI files.
47. Add `check:ui` coverage for `pages.js`, route helper imports, mojibake, and dashboard/chat render smoke.
48. Add a lightweight click-wiring smoke for critical buttons that verifies handler existence and no missing imports.
49. Add release checklist items for manual UX smoke: route persistence, Duo toggle, strategy load, live run recovery, project rail selection.
50. Add a small UX diagnostics panel hidden under debug that reports route key, selected project, chat key, active run project, Duo session project, and scope.

## Implementation Order

1. Stabilize correctness polish first: Strategy crash gates, mojibake check, route helper, route persistence, in-app refresh.
2. Simplify visible structure: left rail everywhere, legacy Duo redirect, route summary/details split.
3. Improve chat control clarity: context chips, risk-based buttons, disabled reasons, pending states.
4. Improve live execution UX: unified live strip, recovery, durable partial output.
5. Polish project/detail surfaces: rail badges/filters, project detail classes, next safe action.
6. Expand regression checks and release checklist.

## Acceptance Criteria

1. The user can always answer "where am I, who am I talking to, what project is active, what will the next message do?"
2. The user can involve two agents without leaving the main chat or seeing a second incompatible Duo product.
3. No primary button is ambiguous about whether it only inserts text, runs a read-only check, or writes/executes.
4. Long-running work always shows a live phase, last event, elapsed time, and stop path.
5. Reload preserves the active route and does not make the app look idle while backend work is running.
6. Strategy, Plans, Graph, Project, and Orchestrator contexts are visible in chat before sending.
7. UI source cannot ship obvious mojibake or missing hook imports again.
