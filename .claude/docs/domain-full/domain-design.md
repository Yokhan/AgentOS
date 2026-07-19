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

## Core Principles

### Token-First
NEVER hardcode visual values. Always bind to tokens/variables:
- Colors → design token or CSS variable (never raw hex in code or Figma)
- Typography → text style or font token (never raw fontSize)
- Spacing → spacing token (never raw px)
- Border radius → radius token (never raw value)
- Shadows → effect style (never raw box-shadow)

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

## 8-Phase Design Pipeline

Every design task follows this pipeline. No phase may be skipped.

| Phase | Name | What |
|-------|------|------|
| 0 | **CONTEXT** | User journey, design language, device/viewport |
| 1 | **ANALYZE** | 5-lens: Art Direction, UX, UI, Flow, Behavior |
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

## Self-Audit Gate (after every creation step)

```
[ ] Every color bound to a token/variable? (no raw hex)
[ ] Every text has a text style applied? (no raw font settings)
[ ] Every border radius bound to a token? (all corners)
[ ] Every spacing/padding bound to a token?
[ ] Every effect uses an effect style?
[ ] Every container has layout mode set?
[ ] No fixed sizing where HUG/FILL is appropriate?
[ ] No placeholder text? (real content or realistic data)
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
