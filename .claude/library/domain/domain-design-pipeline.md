# Domain: Design (Product/UX/Visual/Game) — Evidence-Based Guard

## NEVER Recommend (Anti-patterns)
1. **Feature Factory** — Measuring success by features shipped, not outcomes. Instead: outcome-driven development with retention metrics.
2. **Dark Patterns** — 97% of top EU sites use deceptive UI (EC 2022). Erodes trust, now carries fines. Instead: transparent opt-ins.
3. **Accessibility Overlays** — Only 2.4% of disabled users find them effective; they break assistive tech. Instead: semantic HTML, ARIA, keyboard nav.
4. **A/B Testing Cargo Cult** — Peeking inflates false positives from 5% to 26%. Instead: pre-register, calculate sample size, use sequential testing.
5. **Building What Users Literally Request** — Users can't articulate solutions ("faster horse"). Instead: Jobs-to-be-Done framework.
6. **Loot Box Gambling** — Exploits variable-ratio reinforcement; illegal in Belgium/Netherlands. Instead: transparent direct purchase.
7. **Tutorial Info Dump** — Front-loading all mechanics violates cognitive load limits. Instead: progressive disclosure, one concept at a time.
8. **Ignoring Cognitive Load** — Miller's Law: 7+/-2 chunks. Hick's Law: more options = slower decisions. Instead: chunk info, progressive disclosure.
9. **Using 4+ Fonts** — Creates visual chaos, resets cognitive processing ~20%. Instead: max 2-3 typefaces.
10. **Pixel-Perfect Obsession** — Chasing pixel alignment kills shipping speed. Instead: define visual tolerance, focus polish on high-impact points.
11. **Insufficient Color Contrast** — 96.3% of top 1M pages fail WCAG (WebAIM). DETECT: contrast ratio <4.5:1 for normal text. Instead: WCAG AA minimum, test with contrast checker.
12. **Ignoring Fitts's Law** — Tiny touch targets and distant sequential actions increase errors. Apple HIG minimum: 44x44px. Instead: size targets proportional to importance.
13. **Anti-Fun Exceeding Fun** — Game mechanics where negative experience for opponent > positive for user (hard CC, stunlocks). Instead: ensure fun-for-user outweighs anti-fun-for-opponent (Zileas).
14. **Balance by Spreadsheet Only** — Math alone cannot predict player behavior; players find exploits designers never anticipated. Instead: spreadsheets for foundation + playtesting for truth.
15. **Mandatory Account Before Value** — Requiring signup before users experience product value. Each friction point loses 3-8%. Instead: let users try core value first, sign up after.

## ALWAYS Apply (Principles)
1. **Continuous Discovery** — Weekly customer interviews + assumption testing keeps decisions evidence-based.
2. **5-User Iterative Testing** — 5 users per round find ~85% of major problems; iterate and retest.
3. **Accessible-First Design** — Integrate WCAG from first sprint. Retrofitting costs 10x more. 15-20% expanded market.
4. **Flow Channel Design** — Balance challenge to skill (Csikszentmihalyi). Clear goals, immediate feedback, adaptive difficulty.
5. **Core Loop First** — If the fundamental 30-second loop isn't compelling, no polish saves it.
6. **Performance as UX** — 53% abandon at >3s load (Google). Each second costs ~7% conversions.
7. **Behavioral Over Attitudinal** — Weight what users DO (analytics) over what they SAY (surveys). Say-do gap is massive.
8. **Error-State-First Design** — Design error states and empty states before the happy path.
9. **Visual Hierarchy via Contrast/Space** — Whitespace increases comprehension 20%. Use size, weight, color to guide attention.
10. **Subtractive Design** — Remove until only essential remains. A game/product is done when nothing can be taken away.
11. **Keyboard-First Interactive Design** — All interactive elements must work with keyboard before adding mouse/touch. Serves power users + screen readers + motor-impaired (WCAG 2.1 SC 2.1.1).
12. **Respect prefers-reduced-motion** — ~35% of adults over 40 have vestibular disorders. Check the media query; provide static alternatives for all animations.
13. **Negative Feedback Loops in Games** — Catch-up mechanics prevent snowball effects that drive losing players to quit. Mario Kart's item distribution is the gold standard.
14. **Cognitive Load Budget: 4 chunks max per decision** — Cowan's updated limit (2001) is 4+/-1, not Miller's 7. Design screens around 3-5 actionable items, not 7-9.
15. **Game Juice = Layered Feedback** — Visual + audio + haptic on every meaningful action. Audio alone is the cheapest high-impact improvement. Always include "reduce motion" option.

## For Details
See `brain/03-knowledge/domains/product-design-ux.md`, `graphic-design-and-writing.md`, `game-design.md` for full practices.

---

# Design Pipeline — Token-First, Component-First, Compose-First

> Universal design production rules. Project-specific details (component IDs, brand tokens)
> belong in `project-figma-workflow.md`, not here.
> For design-system work, also read `.claude/library/domain/domain-design-system.md`.

## Core Principles

### Durable Design Context
Before design/UI work, check root `DESIGN.md` when present.

- `tasks/goal.md` owns user value, product/business priority, KPI impact, dependencies, and current step.
- `DESIGN.md` owns visual direction, tokens, typography, layout scale, component behavior, and visual guardrails.
- If `DESIGN.md` exists, preserve it unless the user explicitly asks to update visual direction.
- If `DESIGN.md` is missing in a product with UI work, create or request a starter design context before broad visual changes.
- Never overwrite a project `DESIGN.md` during template sync or design refresh without explicit product-owner approval.

### Register Gate

Choose the design register before judging or changing UI:

- Product register: apps, dashboards, admin tools, settings, forms, workflow surfaces, commerce systems, and game systems. Design serves the user's current job, scan speed, error reduction, retention, activation, conversion, loyalty, support load, or another app-specific KPI.
- Brand register: landing pages, campaigns, portfolios, venues, launches, editorial pages, and public marketing. Design carries recognition, memorability, trust, proof, offer clarity, conversion, and loyalty.
- Mixed register: product surfaces with public brand pressure. The product user's next action and app-specific KPI still outrank visual novelty.

Do not review product UI as if it were a campaign page. Do not review a brand surface as if memorability and proof were optional.

### Critique Ordering

For critique and audit, inspect the actual UI, screenshot, or flow before reading deterministic validator output when possible.

Human judgment before deterministic findings:

- First decide whether the surface serves the current user job and business outcome.
- Then run the subtraction gate and register-aware review.
- Then use validator output as evidence for concrete fixes.

### Token-First
NEVER hardcode visual values. Always bind to tokens/variables:
- Colors → design token or CSS variable (never raw hex in code or Figma)
- Typography → text style or font token (never raw fontSize)
- Spacing → spacing token (never raw px)
- Border radius → radius token (never raw value)
- Shadows → effect style (never raw box-shadow)

If a required value has no token, stop and add/request the token. Do not create one-off visual values inside molecules, organisms, templates, or screens.

### Component-First
NEVER build from raw shapes or primitives when a component exists:
- Search design system / component library first
- Use instances/imports of existing components
- Only create new when no match exists AND the pattern will be reused

### Composition Over Creation
Build screens by composing existing components, not by drawing new ones:
- Assemble from component instances
- Override props/slots/variants — don't recreate from scratch
- If a component doesn't support what you need → extend it, don't bypass it
- For accepted future behavior, extend the component with an end-state
  skeleton: named slots, states, handlers, props, events, or feature flags can
  be 1% callable with honest stubs, but the component contract should not omit
  known final capabilities.
- Do not show fake completed behavior to users. Stubbed interactions must be
  dev-only, explicitly unavailable, or safely no-op until implemented.
- When a later UI layer supersedes an earlier one, remove or replace stale
  hidden panels, disabled controls, obsolete stories, old routes, skipped
  checks, and release-only harnesses before claiming the next readiness level.

### Screen Anatomy First

Before placing content on any full screen, name and build the screen layers:

1. Root frame: viewport/min-height, isolation, overflow, base surface, and base text color.
2. Base background: flat fill, gradient, image/media slot, or another approved surface token.
3. Background composition: absolute decorative or media layer independent from content spacing.
4. Content frame: safe-area, responsive padding, max-width/grid, column model, and allowed organisms.
5. Overlay layer: modal, drawer, toast, sticky action, or temporary system layer only when the scenario needs it.

Headers, heroes, forms, service rails, product cards, docs content, and account panels live inside the content frame. Background images, gradients, decorative shapes, and texture layers must not define content spacing or grid behavior.

Frame rule:

- Visible bounded surfaces (glass, cards, panels, modals, framed media) need internal padding, border/radius/effect tokens, and a declared content slot.
- Edge-to-edge or naked content sections use the page grid/content frame directly and do not get fake card padding.
- If a story or screen cannot name the root frame, background, content frame, and overlay policy, stop before styling.

### Subtraction-First
Before improving a screen, decide what should be removed, hidden, collapsed, or moved. Do not start by adding panels.

Every screen serves exactly one current user job. Examples:
- Dashboard: scan, triage, decide, act.
- Form/editor: create, edit, submit, recover.
- Commerce/gear: compare, select, buy, equip.
- Settings/admin: configure, grant, revoke, audit.
- Content/log/codex: review history, not drive the primary loop.
- Game run/combat: continue, survive, descend, return.
- Game camp/town: spend, cleanse, upgrade, prepare.

Add only after the subtraction pass names what to keep, remove, collapse, and move.

## 8-Phase Design Pipeline

Every design task follows this pipeline. No phase may be skipped.

| Phase | Name | What |
|-------|------|------|
| 0 | **CONTEXT** | User journey, product/business priority, `DESIGN.md`, register, screen anatomy, device/viewport |
| 1 | **ANALYZE** | 5-lens + UI Subtraction Gate |
| 2 | **REFERENCE** | Find gold-standard, deep-inspect structure |
| 3 | **BOM** | Bill of Materials — list ALL component instances needed |
| 4 | **DISCOVER** | Query available tokens, styles, components |
| 5 | **COMPOSE** | Create from instances, bind all values to tokens |
| 6 | **VALIDATE** | Screenshot + compare + self-audit gate |
| 7 | **ITERATE** | Fix deviations, re-validate |

### Phase 1 Detail: 5-Lens Analysis
1. **Art Direction** — brand guidelines match? Tone, mood, identity?
2. **UX** — can user accomplish goal? No dead ends? Clear hierarchy?
3. **UI** — all values from tokens? Consistent spacing? Systematic?
4. **Flow** — where from → what they see → where they go?
5. **Behavior** — all states covered? (see State Coverage below)

## UI Subtraction Gate

Before adding UI, answer:

1. Can the user make a decision from this element right now?
2. Is this information needed every second, or only on demand?
3. Does it duplicate another signal?
4. Does it belong to this mode?
5. Is it competing with the primary action?
6. Can it become a badge, drawer, tooltip, details section, or nav badge?
7. Would removing it make the next action clearer?

General rules:

- Keep the current mode's best next action before long lists.
- Collapse secondary diagnostics, explanations, recommendations, and history until needed.
- Move cross-mode information to the mode where it becomes actionable.
- Avoid dead disabled buttons in primary action zones; explain unavailable actions where the user can fix them.
- Reset scroll on major tab or mode changes when old scroll position would hide the new primary action.

Mobile/game additions:

- Keep main tap targets at least 52px high.
- Use disclosure for long progression matrices.
- Do not show full wallet on run screens unless spending is possible there.
- Do not show meta-upgrade advice inside an active run unless there is a direct action.
- One danger state gets one primary textual signal; the rest should be visual treatment.

Design reviews must include: Keep, Remove, Collapse, Move, Add only after subtraction.

## Design Work Modes

Use the smallest mode that matches the request instead of running the whole pipeline at maximum weight:

- `shape`: clarify product job, surface, constraints, and visual lane before edits.
- `craft`: implement a confirmed UI change end to end.
- `audit`: technical quality scan for accessibility, responsiveness, token drift, and anti-patterns.
- `critique`: UX/design review with severity, user impact, and next action.
- `distill`: remove, collapse, or move UI before adding anything.
- `harden`: edge cases, i18n, overflow, loading/error/empty, long data, slow/offline states.
- `polish`: final alignment, spacing, density, hierarchy, and visual consistency pass.
- `adapt`: mobile/desktop viewport and touch-target adaptation.
- `clarify`: labels, error copy, instructions, and support/empty state text.
- `typeset/layout`: typography and spatial rhythm fixes.

Modes do not weaken the production bar. They only pick the most direct path to the same register, token, component, subtraction, hardening, and rendered-evidence gates.

Mode output must name the chosen register, product user, user/business outcome, rendered evidence or residual risk, confidence, and the main doubt for M+ work.
For full-screen work, it must also name the screen anatomy layers or explicitly mark the missing layer as a blocker.

## State Coverage (mandatory)

Every interactive element must have these states designed:

| State | When |
|-------|------|
| Default | Resting |
| Hover | Mouse over |
| Active/Pressed | Being clicked |
| Focus | Keyboard navigation |
| Disabled | Non-interactive |
| Loading | Async in progress |
| Error | Validation / error |
| Empty | No data |

Not all states apply everywhere. But the designer must DECIDE which apply — not ignore them.

## Browser And Visual Hardening Gate

For M+ UI work, product surfaces, forms, dashboards, app shells, and design-system primitives, verify real rendered behavior before closeout:

- Capture or inspect desktop and mobile viewports.
- Check important controls with `getBoundingClientRect()` or equivalent geometry evidence.
- Stress long text, long words, large numbers, empty data, many items, and short labels.
- Verify loading, error, empty, disabled, focus, hover, active, and default states where applicable.
- Check slow/offline/API error behavior when the UI depends on network data.
- Check 200 percent zoom or text scaling when feasible.
- Respect `prefers-reduced-motion` for animated surfaces.
- Document residual doubt when a browser, Storybook, or screenshot check is unavailable.

Use Playwright screenshot snapshots, Storybook visual tests, Chromatic, or equivalent visual regression only when the project already uses them or the surface is high-value enough to justify the added tool.

## Self-Audit Gate (after every creation step)

```
[ ] Every color bound to a token/variable? (no raw hex)
[ ] Every text has a text style applied? (no raw font settings)
[ ] Every border radius bound to a token? (all corners)
[ ] Every spacing/padding bound to a token?
[ ] Every effect uses an effect style?
[ ] Every container has layout mode set?
[ ] Full-screen work declares root frame, background composition, content frame, and overlay policy?
[ ] No fixed sizing where HUG/FILL is appropriate?
[ ] No placeholder text? (real content or realistic data)
[ ] Known future behavior represented as 1% callable component slots/states/handlers when needed?
[ ] Screenshot taken and visually verified?
```

If ANY fails → fix before moving to next component.

## Atomic Design Hierarchy

```
Tokens     → Color, typography, spacing, radius, shadows
Atoms      → Icon, Logo, Avatar, Divider, Badge, Label
Molecules  → Button, Input, Tag, Toggle, SearchBar
Organisms  → Header, Footer, Card, Sidebar, Form
Templates  → Page layouts (organisms in grid)
Screens    → Templates + real data + navigation
```

**Build order**: always Tokens → Screens. Never skip levels.

Higher layers must expose a dependency trace:

- Molecule → tokens + atoms used.
- Organism → tokens + atoms + molecules used.
- Template → organisms + layout tokens used.
- Screen → template + real data + navigation state.

If the trace cannot be named, the component is probably bypassing the system.

## Rendered Geometry Gate

For web/product UI, token references must be verified against rendered output, not only source text.

Check important stories or pages with browser automation:

- `getComputedStyle()` for color, typography, radius, spacing, and motion tokens.
- `getBoundingClientRect()` for control heights, gaps, card sizing, and layout constraints.
- Mobile and desktop viewport screenshots for overflow, overlap, clipped text, and parent grid/flex stretch.

This is mandatory for design-system primitives, forms, headers, app shells, dashboards, and service gateway screens.

## Figma MCP Specifics (when available)

### Two-Tool Workflow
- **Figma MCP (`use_figma`)**: structure, instances, token binding, layout
- **Chrome DevTools console**: font loading workarounds, text overrides

### Key Rules
- `importComponentByKeyAsync` fails on local components → use `figma.getNodeById(id).createInstance()`
- Always discover tokens first (`getLocalVariablesAsync`, `getLocalTextStylesAsync`)
- Use `textStyleId` to switch fonts without `loadFontAsync`
- `clipsContent = false` on organizational frames, `true` on viewport screens
- Validate with `get_screenshot` after every structural change

### Pre-Creation Gate
Before creating ANY element in Figma:
1. `search_design_system` — does this component already exist?
2. TOKEN lookup — is there a token for this value?
3. Text style lookup — is there a style for this size/weight?
4. If YES → USE EXISTING. If NO → create AND register in tool-registry.

## Cross-References
- `.claude/library/technical/atomic-reuse.md` — code-side reuse protocol (same philosophy)
- `_reference/tool-registry.md` — Design Tokens section for component registry
- `.claude/library/meta/analysis.md` — "Level 0 analysis is never acceptable" applies to design too
