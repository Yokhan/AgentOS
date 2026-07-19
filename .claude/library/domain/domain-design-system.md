# Domain: Design System Production Workflow

## Purpose
Design systems are production contracts, not collections of nice-looking screens.

A usable system must let agents and humans compose product UI without guessing spacing, typography, radius, motion, states, copy density, or component dependencies.

Root `DESIGN.md`, when present, is the project-owned visual contract that agents read before making design decisions. It summarizes the active token/component language without replacing implementation tokens, Storybook, or Figma libraries.

The design system must declare whether each surface is operating in product register, brand register, or mixed register. Product register prioritizes task clarity, state coverage, density, and user/business outcomes. Brand register can carry stronger art direction, but still needs proof, offer clarity, conversion, and loyalty impact.

## Required Layers

1. Foundations: color, typography, spacing, radius, shadow, motion, grid, breakpoints, z-index.
2. Atoms: icons, labels, buttons, inputs, badges, dividers, loaders.
3. Molecules: form rows, field groups, search bars, tabs, segmented controls, nav items, alert rows.
4. Organisms: headers, account panels, service cards, forms, tables/lists, empty/error/loading panels, app shells.
5. Templates: repeatable page layouts with real responsive rules.
6. Screens: real product data and navigation states.

Every higher layer must declare which lower-layer tokens/components it uses. If a needed value has no token, stop and add/request the token instead of hardcoding.

## Screen Anatomy Contract

Every full screen, template, and full-page Storybook/Figma example starts from the same layered frame contract before product UI is placed:

1. Root frame: viewport/min-height, isolation, overflow, base surface, and base text color.
2. Base background: flat fill, gradient, image/media slot, or another approved surface token.
3. Background composition: absolute decorative or media layer independent from content spacing.
4. Content frame: safe-area, responsive padding, max-width/grid, column model, and allowed organisms.
5. Overlay layer: modal, drawer, toast, sticky action, or temporary system layer only when the scenario needs it.

Headers, heroes, forms, cards, product rails, docs content, and account panels live inside the content frame. Background images, gradients, decorative shapes, and texture layers must not define content spacing or grid behavior.

Bounded vs edge-to-edge rule:

- Visible bounded surfaces (glass, cards, panels, modals, framed media) own internal padding, radius, border/effect tokens, and content-slot rules.
- Edge-to-edge content sections use the page grid/content frame directly and must not be wrapped in fake cards just to create spacing.
- Atomic stories show the atom itself without unrelated frames. Molecule/organism/template stories may add only the minimum wrapper needed to demonstrate the real composition contract.

## Contract Tables

Storybook, docs, or Figma must expose tables for:

- Typography roles: display, title, section, body, caption, control, data.
- Spacing: container padding, section gaps, component gaps, inline gaps, form gaps.
- Radius: control, card, modal, shell, pill, media.
- Control sizes: height, min width, touch target, icon size, padding.
- Motion: duration, easing, transform distance, opacity rules.
- Layout: page width, grid columns, mobile padding, desktop padding, safe areas.
- Screen anatomy: root frame, base background, background composition, content frame, overlay layer, and bounded/edge-to-edge usage.

## Composition Trace

Each molecule, organism, and template should have a visible or testable composition trace:

- Component name.
- Tokens used.
- Child components used.
- States supported.
- Responsive behavior.
- Screen anatomy role, when the component participates in a template or screen.
- Known future behavior exposed through end-state skeleton slots, handlers,
  events, states, or feature flags when the product direction is already known.
- Known exclusions.

This prevents hidden raw values and makes review possible without manually measuring everything.

If a known future capability is not implemented yet, expose it as a 1% callable
stub, no-op, dev-only debug signal, or explicit unavailable boundary. Do not
pretend the state is production-ready for product users.

When the component sharpens, retire superseded design-system layers. Remove or
replace obsolete variants, stale stories, disabled controls, hidden panels,
release-only exclusion harnesses, and feature flags that no longer belong to the
final component contract. Keep temporary migration scaffolding only with an
explicit removal condition.

## Rendered Geometry Gate

Static token references are not enough. Run browser or Storybook checks that compare rendered `getBoundingClientRect()` and computed styles against token values for important components.

Catch at least:

- Parent grid/flex stretching compact cards to full height.
- Buttons using wrong control height, padding, radius, hover/focus state.
- Forms missing label/help/error spacing.
- Text overflow or clipped labels on mobile.
- Icon sizes drifting from token.
- Loading/empty/error panels expanding beyond intended template constraints.

## Product UI Completeness

Landing style is not enough for product UI. A production-ready system also needs:

- Basic forms: input, textarea, select, checkbox, radio, switch, validation, fieldset.
- Account/auth surfaces: login, registration, recovery, expired session, logout confirmation.
- Daily app surfaces: header, nav, list, table, detail, settings, subscription, empty state, loading state, error state.
- Service gateway surfaces: app card, download/action card, access status, quick links, return path.
- Docs/help surfaces: article shell, nav, search, related links, callouts.

## Review Questions

- Can a new product page be assembled without inventing styles?
- Is the surface being reviewed in the correct product or brand register?
- Does every non-atomic component depend on lower layers?
- Are all numbers named tokens?
- Does mobile use less chrome and still preserve tap targets?
- Are the states visible in Storybook and testable by an agent?
- Does the design serve the user task or only the brand mood?
