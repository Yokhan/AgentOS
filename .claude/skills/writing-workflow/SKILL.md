---
name: writing-workflow
description: "Write literary, marketing, informational, or communication text through one purpose-first, evidence-aware workflow."
---

# Writing Workflow

Read `.claude/library/technical/writing.md` as the single source of truth.

## Process

1. Select one primary semantic mode from the reader's job.
2. Build the writing contract and source pack, including the target language.
3. Gate on material SOT conflicts or missing facts; offer 2-3 options.
4. Analyze references into concrete properties without copying structure.
5. Plan the final text inventory and produce a functional 1% whole.
6. Sharpen the accepted whole instead of accumulating alternate drafts.
7. Run the mode-specific review, then structural, line, grammar, and channel edits.
8. Verify the production purpose, truth boundary, next action, and active version.

Keep language/editorial, process, domain, and technical profiles separate.
For Russian output, load `.claude/library/technical/russian-writing-profile.md`;
also load the routed Russian correspondence and explanation child profiles;
English domain standards may constrain correctness and architecture but not
Russian voice, syntax, idiom, or line editing.

Treat external tools separately from sources. Without configured access and a
successful response tied to the current artifact, do not claim a Glavred check,
score, warning list, or any other provider result.

Read `.claude/library/technical/writing-mode-profiles.md` for detailed contracts
and readiness examples. Select profile IDs from the writing reference registry
and use the editorial board instead of loading an unverified example corpus.
When product truth depends on code, schemas, versions, commands, or runtime
behavior, also use `.claude/skills/technical-writing/SKILL.md`. Read
`docs/WRITING_WORKFLOW.md` only when changing the workflow architecture.
