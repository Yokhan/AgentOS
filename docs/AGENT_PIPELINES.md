# Agent Pipelines

Shared, agent-neutral workflow definitions for Claude Code, Codex, and future agents.

Agent-specific wrappers may live under `.claude/pipelines/` or `.agents/skills/`, but these definitions are the contract.

## Gates

| Gate | Pass condition | On fail |
| --- | --- | --- |
| `research_done` | Affected files, history, lessons, registry, and risks are summarized | Continue research |
| `product_goal_preserved` | Final outcome, quality bar, current step, dependencies, and out-of-scope items are explicit | Restore or create `tasks/goal.md` |
| `plan_approved` | User approval when risk or size requires it | Wait for user |
| `typecheck_pass` | Relevant typecheck or syntax check passes | Fix before next phase |
| `tests_pass` | Relevant tests pass | Fix or document blocker |
| `review_pass` | Review finds no blocking defects | Address findings |
| `security_pass` | Vulnerability is closed and no new obvious attack surface appears | Rework patch |
| `design_validated` | Screenshot/manual checks pass and design rules are met | Iterate |

## Artifact-Driven Work

When a project already has Spec Kit, litkit, Kiro, AgentOS, or another
artifact-driven flow, use those artifacts instead of inventing a parallel process.
Common artifact chain:

1. Spec: user-visible behavior and acceptance criteria.
2. Plan: architecture, constraints, risks, and verification.
3. Tasks: dependency order plus `[P]` markers for independent work.
4. Implement: parent-owned sequencing, with automatic subagents on useful independent safe splits.

Codex-specific routing and prompts live in `docs/CODEX_FANOUT_PATTERNS.md`.
Codex route selection is made explicit with `node scripts/codex-route-task.js "<task>" --summary --write-state`.

## Change Strategy Overlay

Apply this overlay to the active pipeline when initial reading reveals a causal
system mismatch, or when the fallback triggers through a second failed repair,
recurring workaround, stale-path test, or planned breaking change:

1. During initial reading, run a bounded repair-path check over the affected path
   and direct consumers. If causal system evidence appears, invoke the overlay
   before the first patch. Reroute the original task once with `--discovery-file`
   only when pipeline, risk, or approval authority changes. The second failed
   repair remains the mandatory fallback trigger.
2. Prove the discovery, repeated attempt, or planned-change trigger against a
   named evidence reference, acceptance, or falsifier.
3. Classify project posture and inventory protected contracts with owner, SOT, and impact.
4. Choose destination (`repair|bounded-replace|retire-remove`) separately from transition (`direct-swap|staged-swap|versioned-coexistence|expand-migrate-contract`).
5. Eliminate options that fail product function, data safety, security/privacy, contracts, verification, or recovery constraints.
6. Compare remaining options against one objective evidence baseline, including maintainability, reliability, performance, total cost, and reversibility.
7. Continue automatically only inside the approved change envelope; otherwise notify the client with 2-3 options and a recommendation.
8. Remove superseded paths or time-box transition scaffolding with owner, removal condition, and absence check.

`blockEdits` means "pending a valid Change Strategy decision", not permanent
stoppage. Validate the optional decision JSON, update the same route state with
`--decision-file`, and resume the original pipeline when validation is
unblocked and the decision trigger plus evidence reference match the discovery.
The discovery file is ephemeral router input, never another SOT.

This overlay does not replace the orchestrator or create a parallel task graph.

## Production Product Standard

Use for any real product, design, auth, data, game, docs, deployment, or M+ work.

1. Restore goal
   - Read `tasks/goal.md` when present.
   - If absent for M+ product work, create or propose it.
2. Preserve final outcome
   - State the final user outcome, quality bar, current step, dependencies, risks, and out-of-scope items.
   - Do not use MVP/prototype reasoning unless the user explicitly asks for a disposable experiment.
3. Route and plan
   - Route the task and save the current step to `tasks/current.md`.
   - Plans, audits, and reports use the language of the user's request.
4. Execute bounded step
   - Keep the step small and reversible.
   - Do not introduce fake UX, dead ends, privacy regressions, or architecture shortcuts that conflict with the final product goal.
5. Verify product effect
   - Prove user-visible behavior or contract improvement, not only file changes.
6. Update
   - Update `tasks/goal.md` only when final outcome or quality bar changes.
   - Mark partial work honestly.

## Feature

Use for new capabilities, modules, screens, commands, or workflows.

1. Research
   - Read affected files and neighbors.
   - Check recent git history for those files.
   - Check `tasks/lessons.md`, `PROJECT_SPEC.md`, and `_reference/tool-registry.md`.
   - Classify risk.
   - Output: research summary and approach options.
2. Brainstorm
   - Required for HIGH/CRITICAL risk, optional for MEDIUM.
   - Compare at least 2 approaches with reversibility and test cost.
3. Plan
   - Save a concrete plan for M+ work.
   - Include file architecture, implementation order, risks, Plan B, and test scenarios.
4. Implement
   - Work in small batches.
   - Keep business logic in modules and entry points thin.
5. Test
   - Add or update focused tests.
   - Run relevant checks.
6. Review
   - Look for behavioral regressions, missing tests, and broken boundaries.
7. Closeout
   - Summarize outcome, verification, confidence, and remaining doubt.

## Bugfix

Use for incorrect behavior, failed tests, crashes, or regressions.

1. Research
   - Read the failing code path, callers, tests, recent history, and lessons.
2. Reproduce
   - Create or identify a minimal failing case.
   - Do not fix until the failure is observable.
3. Diagnose
   - Explain root cause and blast radius.
4. Fix
   - Make the smallest root-cause fix that preserves unrelated behavior.
5. Regression test
   - Add a test or smoke check that fails without the fix.
6. Verify and closeout
   - Run checks and record lesson if the bug pattern is reusable.

## Security Patch

Use for vulnerabilities, secrets, auth, injection, permissions, CVEs, or data exposure.

Risk is always HIGH or CRITICAL.

1. Research
   - Identify affected files, data, actors, trust boundaries, and logs.
2. Assess
   - Classify severity and exploitability.
   - State what would prove the patch works.
3. Plan
   - User checkpoint required for CRITICAL or broad changes.
4. Patch
   - Keep the fix narrow. Do not refactor unrelated code.
5. Security verification
   - Prove the exploit path is closed.
6. Regression tests
   - Cover vulnerable and allowed behavior.
7. Closeout
   - Note any rotation, disclosure, or deployment steps.

## Design

Use for Figma, UI, CSS, frontend screens, design systems, game UI, or visual polish.

1. Context
   - Identify user journey, viewport, design language, and constraints.
2. Analyze
   - Use the 5 lenses: art direction, UX, UI, flow, behavior.
3. Reference
   - Inspect comparable gold-standard products or existing system components.
4. Bill of materials
   - List required tokens, components, states, assets, and content.
5. Discover
   - Search design system before creating anything.
6. Compose
   - Build tokens -> components -> screens.
7. Validate
   - Screenshot, responsive checks, contrast, overflow, state coverage, and rendered geometry for important components.
8. Iterate
   - Fix deviations and re-validate.

## Design System

Use for tokens, Storybook, component libraries, atomic design, forms, product UI primitives, and reusable layout contracts.

1. Foundations
   - Confirm token tables for typography, spacing, radius, motion, layout, and control sizes.
2. Atoms and molecules
   - Compose only from lower layers.
   - Add/request tokens before using new visual values.
3. Organisms and templates
   - Record composition traces and responsive behavior.
4. Product surfaces
   - Cover forms, account/auth, service gateway, empty/loading/error, docs/help, and everyday product UI.
5. Rendered checks
   - Use Storybook/browser automation to compare computed styles and bounding boxes against tokens.

## Writing

Use for literary, marketing/advertising, informational, and communication text.

1. Classify
   - Use `scripts/lib/writing-intent.js` to identify action, primary mode, and domain overlays.
   - Channel words do not create a second primary mode; a marketing email remains marketing.
2. Contract
   - Define reader, production purpose, after-state, product/business link, channel, SOTs, voice, and acceptance evidence.
3. Research
   - Load relevant facts, prior text, references, claims, constraints, and gaps.
4. Architect
   - Define the final function/section inventory and mode-specific reader path.
5. Functional whole
   - Produce the smallest complete text that performs the production purpose honestly end to end.
6. Sharpen
   - Increase evidence, examples, scenes, objections, detail, and language resolution in the accepted whole.
7. Review
   - Use a separate reviewer for public, commercial, sensitive, or M+ text; otherwise label the pass as self-check.
8. Release and evolve
   - Verify truth, channel, links, CTA/resolution, active version, and outcome evidence; replace superseded text.

## Technical Writing

Use as a specialization over informational writing, or communication for
incidents, release notes, and migration notices.

1. Contract
   - Identify reader, product/version, OS/shell, deployment shape, document kind,
     active SOTs, and executable acceptance path.
2. Select
   - Run the shared writing policy and record selected reference profile IDs and
     editorial role IDs.
   - Validate template and project-owned registries before loading sources.
3. Inspect
   - Compare existing prose with code, schemas, interfaces, tests, and observed
     behavior; gate material SOT conflicts.
4. Write the functional whole
   - Let the reader complete or understand the production job through the final
     path at the declared depth.
5. Verify independently
   - Separate accuracy, procedure execution, information architecture, and
     technical language lanes. Add API, security, or migration specialists only
     when those contracts apply.
6. Replace and release
   - Remove superseded instructions, verify links/build/rendering, and record the
     lifecycle owner and version boundary.

## Template Maintenance

Use for `AGENTS.md`, `CLAUDE.md`, skills, subagents, hooks, MCP router, setup/sync payloads, and validation scripts.

1. Route
   - Run `node scripts/codex-route-task.js "<task>" --summary --write-state`.
   - Confirm whether AgentOS or project artifacts own the task graph.
2. Boundary check
   - Read `docs/PRODUCT_BOUNDARY.md`, `docs/SAFE_DEFAULTS.md`, and `docs/SUPPORTED_ENVIRONMENTS.md`.
   - Preserve `project-*` overlays and project-owned files.
3. Patch
   - Update Unix and Windows setup/sync paths together.
   - Keep model, effort, sandbox, and approval defaults out of project `.codex/config.toml`.
4. Smoke
   - Run skill, agent, route, hook, and template validation relevant to the change.
5. Sync proof
   - Prove new template-owned files are included in setup and sync allowlists.
6. Closeout
   - State compatibility impact for existing projects and AgentOS workspaces.

## Template Release

Use for release tags, release notes, downstream rollout, and AgentOS rollout coordination.

1. Freeze scope
   - Identify the version and whether the release is patch, minor, or major.
2. Validate
   - Run the release gate from `docs/TEMPLATE_RELEASES.md`.
3. Tag
   - Create a `vX.Y.Z` git tag only after validation is green.
4. Downstream instructions
   - Tell projects to use `scripts/sync-template.sh --from-git --ref <tag>`.
5. AgentOS note
   - If AgentOS is present, it chooses rollout order and records the template tag; it does not replace template sync.

## Closeout Format

For completed work, use:

1. What was incomplete or wrong before.
2. What is true now.
3. What this gives the project or user.
4. What to expect next.
