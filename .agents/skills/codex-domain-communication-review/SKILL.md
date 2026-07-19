---
name: codex-domain-communication-review
description: "Review writing, messaging, marketing copy, positioning, and communication for clarity, evidence, manipulation, corporate speak, and platform fit."
---

# Codex Domain Communication Review

This is an independent review skill, not the writing orchestrator. Generation and
editing belong to `$codex-writing-workflow`.

Read `.claude/library/technical/writing.md` and
`.claude/library/technical/writing-mode-profiles.md`. Also load the selected
profiles from `.claude/library/technical/writing-reference-registry.json` and the
role composition in `.claude/library/technical/writing-editorial-board.md`.

## Process

1. Identify the selected writing mode, target language and its resolution,
   reader, production purpose, channel, and requested after-state. For edit or
   review work, inspect the artifact language before applying a language profile.
2. Check the active SOT, facts, claims, evidence, uncertainty, and source conflicts.
3. Verify the mode-specific gate: narrative function, commercial decision,
   reader task, or interaction state change.
4. Check manipulation, dark patterns, corporate abstractions, omissions, and
   channel mismatch without rewriting facts or strategy silently.
5. Review language/editorial profiles separately from process, domain, and
   technical profiles. A domain standard cannot supply target-language voice or
   syntax. Preserve specific project voice and terminology; deliberate errors and
   AI-detector evasion are not human-voice criteria.
6. Report findings first with impact and concrete revision direction.

Before judging prose polish, verify the final function/section inventory and the
functional progressive whole. At 1% readiness, the complete text must already
perform its production purpose honestly. An outline, stub, TODO list, evidence
slot, or fabricated proof is not a valid slice.

For documentation grounded in code, schemas, versions, commands, or runtime
behavior, use `$codex-technical-writing-review` for acceptance.
