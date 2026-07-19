---
name: domain-communication-review
description: "Independently review literary, marketing, informational, and communication text for purpose, truth, clarity, ethics, and channel fit."
---

# Domain Communication Review

This is an independent review skill, not the generation workflow. Read:

- `.claude/library/technical/writing.md`
- `.claude/library/technical/writing-mode-profiles.md`
- `.claude/library/technical/writing-reference-registry.json`
- `.claude/library/technical/writing-editorial-board.md`
- project-owned facts, voice, claims, legal constraints, and source hierarchy

Use `.claude/skills/writing-workflow/SKILL.md` or `$codex-writing-workflow` to
create or rewrite text. A self-check by the writer is not independent evidence.
Use `.claude/skills/technical-writing-review/SKILL.md` when product truth depends
on code, schemas, versions, commands, or runtime behavior.

## Evidence Protocol

1. Separate project facts, external evidence, assumptions, preferences, and
   reviewer hypotheses.
2. Verify current or high-stakes claims against authoritative sources before
   recommending them.
3. Do not preserve inherited percentages, benchmarks, platform stereotypes, or
   named-framework rules without source and context checks.
4. Treat readability scores, word lists, paragraph ratios, and formulas as
   diagnostic signals, never universal acceptance criteria.
5. Do not invent reader research, conversion evidence, testimonials, citations,
   legal conclusions, or independent-review status.

## Review Sequence

1. **Contract** - identify action, primary mode, target language and its
   resolution, reader, purpose, after-state, product/business link, channel,
   SOTs, and acceptance evidence. Inspect artifact language before line editing.
2. **Truth** - check facts, claims, canon, attribution, uncertainty, omissions,
   and conflicting sources.
3. **Functional whole** - verify that the complete text performs its production
   purpose at the declared progressive readiness. An outline or evidence slot is
   not the deliverable.
4. **Mode gate** - apply only the primary mode's acceptance criteria.
5. **Structure** - test dependency order, causality, emphasis, and next action or
   resolution.
6. **Language** - apply only the resolved language/editorial profiles; process,
   domain, and technical standards cannot supply voice or syntax. Find ambiguity,
   corporate abstraction, cliche, accidental
   repetition, inconsistent terminology, buried actors, and tone mismatch.
7. **Channel** - check format, length, hierarchy, accessibility, links, rendering,
   and recipient context.
8. **Ethics and harm** - identify manipulation, fake urgency, hidden conditions,
   blame, pressure, unsafe advice, or unsupported certainty.

## Mode Questions

### Literary

- Does the scene or passage create the intended experience and meaningful change?
- Are causality, motivation, point of view, information access, continuity,
  pacing, dialogue function, and story facts coherent?
- Does language serve the accepted voice rather than imitate a reference?

### Marketing And Advertising

- Is the audience/category situation real and the offer understandable?
- Are outcomes, features, conditions, proof, and uncertainty distinguished?
- Does the CTA lead to an available path and support the relevant KPI?
- Are urgency, scarcity, social proof, pricing, and targeting honest and lawful?

### Informational

- Can the reader find the answer or complete the stated task correctly?
- Are prerequisites, versions, warnings, verification, failure, and recovery clear?
- Are claims supported at the level required by freshness and risk?

### Communication

- Can the recipient tell what happened, what matters, what to do, who owns it,
  and when the next action or update occurs?
- Does tone match authority, relationship, consequence, and uncertainty?
- Are reply, escalation, and support paths available where needed?

## Output

Report findings first, ordered by severity. For each finding include:

- evidence from the text or active SOT;
- reader/product/business impact;
- the violated contract or mode gate;
- concrete revision direction without inventing replacement facts.

Then state remaining evidence gaps, residual risk, and whether the pass was an
independent review or only a self-check.
