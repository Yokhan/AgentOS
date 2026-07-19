# Design Command Modes

Use this reference when a design task is M+ risk, asks for one of the command
modes, or needs a production-quality UI decision without loading the entire
design manual into hot context.

## Register Gate

Choose the design register before judging or changing UI:

| Register | Use when | Primary quality test |
| --- | --- | --- |
| Product register | App, dashboard, admin, tool, settings, commerce, game system, form, workflow | The user can complete the current job faster, with less error and less support load. |
| Brand register | Landing, campaign, portfolio, venue, product launch, editorial, public marketing | The surface is memorable, ownable, trustworthy, and tied to a clear conversion or loyalty outcome. |

Rules:

- Product register uses earned familiarity: standard controls, scan density, restrained hierarchy, and low-friction flows.
- Brand register may use stronger art direction, but the first viewport must still prove the offer, object, place, person, or product.
- If both registers apply, the product user's next action and app-specific KPI win over visual novelty.
- Technical polish matters only when it protects or improves user experience, revenue, activation, conversion, retention, loyalty, support load, or another app-specific KPI.

## Mode Matrix

| Mode | Use when | Must produce | Verify with |
| --- | --- | --- | --- |
| `shape` | Intent, audience, KPI, or visual lane is unclear | A compact brief: job, surface, register, constraints, quality bar, open questions | User confirmation or explicit assumptions before edits |
| `craft` | The brief is clear and implementation is requested | Working UI composed from tokens and components | Browser/screenshot, responsive, state, and geometry evidence |
| `audit` | The user asks to check a UI or design system | Defects ordered by user/business impact and evidence | Static checks plus rendered checks where possible |
| `critique` | The user wants design judgment, not just lint | Keep, Remove, Collapse, Move, Add-after-subtraction, with severity | Human judgment first, validator output second |
| `distill` | A screen feels bloated, noisy, or unfocused | A subtraction plan before any additions | One current user job and primary action remains clear |
| `harden` | UI is close but fragile | Edge-case matrix for data, i18n, states, network, motion, and zoom | Stress fixtures or manual browser checks |
| `polish` | Product works but lacks finish | Small visual and interaction fixes tied to hierarchy and flow | Before/after screenshots or geometry/style checks |
| `adapt` | Viewport, device, or input mode changes | Mobile/desktop/touch/keyboard adaptation plan or implementation | Target viewport checks and tap/keyboard target checks |
| `clarify` | Copy, labels, errors, or empty states are unclear | User-facing text that reduces hesitation and support load | Read-through in context, error and empty-state checks |
| `typeset/layout` | Typography, rhythm, density, or alignment is weak | Type scale, spacing, line length, and layout corrections | Rendered text, overflow, and hierarchy checks |

## Mode Details

### shape

- Name the real product user and the action they need to take.
- Name the app-specific KPI or business outcome affected by the surface.
- Identify the register: product, brand, or mixed.
- Read root `DESIGN.md` when present; if missing, ask only the minimum visual questions needed to proceed.
- Output assumptions explicitly when the user wants immediate execution.

### craft

- Use system -> tokens -> components -> screens.
- Search existing tokens/components/styles before creating new ones.
- For full screens, name the root frame, base background, background composition, content frame, and overlay policy before placing UI.
- Keep the current mode's best next action before long lists or secondary diagnostics.
- Verify rendered output. If a browser or screenshot is unavailable, state the residual risk.

### audit

- Start with user task fit and product/business impact.
- Check token/component reuse, accessibility, responsive behavior, state coverage, overflow, and geometry.
- Include deterministic validator findings, but do not let them replace design judgment.

### critique

- Inspect the actual UI, screenshot, or flow before reading automated findings when possible.
- Lead with the highest-impact defect.
- Use the review shape: Keep, Remove, Collapse, Move, Add only after subtraction.
- Give a next command suggestion such as `distill`, `harden`, `polish`, or `craft`.

### distill

- Declare the one current user job for the screen.
- Remove, hide, collapse, or move elements that are not actionable now.
- Convert secondary information to badges, drawers, tooltips, details sections, or nav badges.
- Do not add panels until the subtraction pass is complete.

### harden

- Stress long localized strings, long words, large numbers, empty data, many items, and short labels.
- Cover loading, error, empty, disabled, focus, hover, active, and default states where applicable.
- Check slow/offline/API error behavior for network-dependent UI.
- Check zoom or text scaling when feasible.
- Respect reduced-motion preferences for animated surfaces.

### polish

- Fix hierarchy, alignment, spacing, rhythm, density, icon sizing, and control affordance.
- Avoid decorative novelty that weakens the product user's next action.
- Keep changes reversible and tied to a concrete user/business effect.

### adapt

- Preserve the primary action across mobile, desktop, touch, keyboard, and reduced-motion contexts.
- Preserve screen anatomy across viewports: the content frame may change grid and padding, but background composition cannot become the layout source.
- Keep mobile main tap targets at least 52px high unless the platform design system has a stricter standard.
- Reset scroll on major mode changes when stale scroll would hide the new primary action.

### clarify

- Use product-approved language, not internal implementation terms.
- Write error and empty-state text that tells the user what happened and what they can do next.
- Reduce support load without adding instructional clutter to the primary action zone.

### typeset/layout

- Use stable tokenized type roles, spacing, widths, and grid constraints.
- Avoid viewport-scaled font sizes and negative letter spacing.
- Check that the longest word and realistic localized text fit without overlap or clipped controls.

## Output Contract

Design plans, audits, and final reports must:

- match the user's language;
- name the product user and user/business outcome;
- state the chosen register and mode;
- name screen anatomy layers for full-screen work;
- report rendered evidence or residual risk;
- state confidence and the main doubt.
