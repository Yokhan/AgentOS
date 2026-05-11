# AgentOS chat and execution overflow plan

## Goal
Make long orchestration runs readable and responsive when the chat has many PA command batches, blocked delegations, and live execution events.

## Dependencies
1. Keep stream performance fixes from `0.3.24`: byte-offset polling and batched stream commits stay as the base.
2. Reduce rendered DOM before visual polish: old PA run cards and oversized waiting panels must collapse by default.
3. Keep full data recoverable: collapsed UI can hide noise, but copy/details must still expose the underlying rows.
4. Preserve operator decisions: `needs_user` cards stay visible, but duplicates and overflow must be grouped.
5. Add release gates: static checks must prevent regressions back to unbounded PA trace rendering and unbounded execution waiting panels.

## Phase 1 - Chat overflow control
- Collapse older PA command batches by default.
- Keep the latest active/recent batches expanded enough to debug current work.
- Add a transcript render cap with a visible "show all rendered messages" escape hatch.
- Keep `load older` history behavior intact.

## Phase 2 - Execution map overflow control
- Group the `needs_user` panel by project/status/action instead of rendering every delegation card as a full block.
- Show top blockers first, then an overflow counter.
- Keep approve/reject/retry/status actions available on visible items.
- Add compact map diagnostics: visible events, hidden heartbeat/state samples, waiting overflow.

## Phase 3 - UI stability
- Ensure the main stage owns scroll, not the chat rail.
- Avoid horizontal/vertical clipping by giving execution tracks a bounded scroll viewport.
- Keep selected event details outside the dense track.

## Phase 4 - Regression gates
- Add a UI overflow static check for compact PA traces, transcript render cap, grouped waiting panel, and bounded execution map viewport.
- Include it in `npm run check:ui`.

## Phase 5 - Verification
- Run JS syntax checks.
- Run `npm.cmd run check:ui`.
- Run Rust tests if backend changes are made.
- If green, bump version and build updater only after the UI pass is complete.
