---
name: codex-product-ux-audit
description: "Audit product UX flows for usefulness, entry/return paths, auth/session behavior, dead ends, responsive states, docs ingress, and real user value. Use when checking whether a product surface actually works for users."
---

# Codex Product UX Audit

Read:

- `.claude/library/product/production-product-standard.md`
- `.claude/library/domain/domain-design-pipeline.md`

## Flow Audit

Trace the user's path:

1. Entry: where the user starts and which action they choose.
2. Auth/session: logged out, logged in, expired, logout, and recovery states.
3. Useful destination: what concrete value the user gets.
4. Return path: how the user gets back without losing context.
5. Dead ends: empty screens, fake dashboards, missing CTAs, blocked permissions, broken docs links.
6. Mobile and desktop: layout, touch targets, text fit, scrolling, and visible next step.
7. Error handling: network/API failure, no data, access denied, and loading.

## Evidence

Do not call a flow working from HTTP status alone. Use browser automation, screenshots, route checks, API contract checks, or deployment smoke appropriate to the surface.

## Report

Findings first, severity ordered. Include:

- User impact.
- Reproduction path.
- Expected useful behavior.
- Verification gap.
- Suggested smallest reversible fix.
