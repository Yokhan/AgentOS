---
name: codex-writing-workflow
description: "Plan, write, edit, and evolve literary, marketing/advertising, informational, and communication text through one purpose-first workflow with truthful progressive JPEG delivery."
---

# Codex Writing Workflow

Read `.claude/library/technical/writing.md` first. It is the shared SOT.

## Process

1. Select one primary mode by meaning: literary, marketing/advertising,
   informational, or communication.
2. Define the reader, production purpose, after-state, product/business link,
   target language, channel, active SOTs, voice contract, and acceptance evidence.
3. Load only relevant project context. If facts are missing or SOTs conflict in a
   way that changes meaning, stop and ask with 2-3 options.
4. Analyze references into concrete writing properties; do not copy their
   structure or imitate a named author mechanically.
5. Plan the complete final function/section inventory.
6. Produce a truthful functional 1% whole, then sharpen it in place. Planning,
   TODOs, evidence slots, and fragments are preparation, not product evidence.
7. Apply the selected mode profile from
   `.claude/library/technical/writing-mode-profiles.md`.
   Select explicit profile IDs from
   `.claude/library/technical/writing-reference-registry.json` and apply the role
   composition in `.claude/library/technical/writing-editorial-board.md`.
   Treat `languageProfiles`, `processProfiles`, `domainProfiles`, and
   `technicalProfiles` as separate authority groups. For Russian output, load
   `.claude/library/technical/russian-writing-profile.md`, plus the routed
   `russian-business-correspondence.md` and
   `russian-explanation-and-persuasion.md` child profiles. Never let an English
   domain standard determine Russian voice, syntax, idiom, or line editing.
8. Run an independent review for public, commercial, sensitive, or M+ text.
   Independence requires a separate read-only reviewer and genuine child trace;
   otherwise label the pass `self-check`, not `independent review`.
9. Replace obsolete passages and stale drafts, verify the channel, and report the
   declared readiness level honestly.

## Gates

- Never invent facts, proof, citations, testimonials, statistics, capabilities,
  deadlines, personal experience, or canon.
- Never add deliberate errors or arbitrary quirks to appear human.
- Never claim or optimize for AI-detector evasion.
- Never apply a marketing formula to literary, informational, or service text.
- Never call an outline or stub a progressive product slice.
- Never let the writer's self-review stand in for independent acceptance evidence.
- Never present an organization, regulator, vendor guide, or documentation
  framework as an authorial model for the target language.
- Never claim an external service ran, invent its score, or paraphrase a provider
  response unless the route reports configured access and the current artifact
  has a successful recorded response. `glavred-api:not-configured` means the
  agent may use public methodology manually but must report that no API check ran.

Read `docs/WRITING_WORKFLOW.md` when changing this workflow. It records the LitAI
adaptation and ownership decisions without becoming a second behavioral SOT.

When truth depends on code, schemas, interfaces, versions, commands, or a runtime
environment, load `$codex-technical-writing`. Technical writing is an overlay on
informational or communication mode, not a fifth semantic mode.
