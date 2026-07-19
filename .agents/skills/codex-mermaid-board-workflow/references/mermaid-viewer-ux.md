# Mermaid Viewer UX

Use these rules when a Mermaid board is too large for a static markdown preview.

## Required Controls

A project-control viewer should include:

- pan by dragging the canvas;
- wheel zoom toward the cursor;
- `Fit` / `Вписать` button;
- `1:1` actual-size button;
- zoom out, zoom percentage/reset, zoom in;
- SVG download/export;
- status text that says whether render succeeded;
- full-height canvas with hidden body overflow so the board itself moves, not the page.

## Language Rule

Use the project working language for visible board labels and viewer controls. If the project/user works in Russian, the board labels, controls, status, and comment cards should be Russian. Keep internal IDs ASCII.

## Interaction Pattern

Use SVG `viewBox` changes for pan and zoom instead of scaling a bitmap. This keeps text sharp and makes SVG export useful.

Recommended state:

```text
svg
originalViewBox
fitViewBox
viewBox
zoom
dragging
dragStartX / dragStartY
startViewBox
```

Core behavior:

1. Render Mermaid to SVG.
2. Parse the original SVG `viewBox`.
3. Compute a `fitViewBox` from canvas aspect ratio.
4. Set SVG width/height to 100% and control the visible region by changing `viewBox`.
5. On wheel, zoom around cursor position.
6. On pointer drag, translate `viewBox`.
7. On resize, recalculate fit.

## Source Rule

The `.mmd` remains canonical. HTML viewers may embed a copy for local `file://` convenience, but the canonical board file must be updated first and treated as the source of truth.

## Borrowed Pattern

This viewer pattern comes from `ai_audit_mermaid_full/viewer`: Russian UI, warm board canvas, panning, zoom controls, fit/1:1, SVG export, status text, and optional inspector/drilldown behavior.