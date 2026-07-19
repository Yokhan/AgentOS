# Writing Mode Profiles

Technical documentation is not a fifth profile. Apply
`technical-writing-profile.md` over informational mode, or over communication
mode for incidents, release notes, and migration notices whose primary job is a
recipient state change.

This file is a child of `.claude/library/technical/writing.md`. The parent owns
the workflow and truth rules; this file provides detailed mode contracts and
readiness examples through progressive disclosure.

Use one primary mode per text. Channel, voice, tone, and formality modify the mode;
they do not replace its production purpose.

## Shared Contract

| Field | Required decision |
| --- | --- |
| Reader | Who receives the text and what do they already know? |
| Job | What must the text enable now? |
| After-state | What changes in understanding, feeling, decision, or action? |
| Product link | Which user outcome or application KPI does this protect? |
| Truth | Which sources authorize facts, claims, terminology, and uncertainty? |
| Channel | Where will the text be read and what constraints apply? |
| Voice | Which observable properties should the language have or avoid? |
| Target language | Which language profile controls syntax, idiom, rhythm, and line editing? |
| Evidence | What proves the text fulfills its purpose? |

The route reports language/editorial, process, domain, and technical profiles as
four separate authority groups. For Russian output, apply
`russian-writing-profile.md`. English domain guidance may shape correctness or
information architecture, not Russian sentence form.
Russian informational, marketing, and communication work also applies
`russian-explanation-and-persuasion.md` when the reader must understand a model
or make a reasoned decision. Russian communication applies
`russian-business-correspondence.md` for executable recipient-state changes.

## Literary

**Input:** story bible, accepted plan, prior state, scene function, characters,
location, chronology, point of view, future consequence, reference properties.

**Architecture:** initial state -> pressure/conflict -> meaningful turn -> changed
state. Other structures are valid when they create the specified experience.

**Review:** causality, motivation, information access, continuity, pacing, concrete
sensory detail, dialogue function, voice, and consequences.

## Marketing And Advertising

**Input:** audience/ICP evidence, category situation, journey stage, offer, price
and conditions, proof, objections, channel, distribution, CTA path, KPI.

**Architecture:** relevant situation -> honest outcome/offer -> mechanism and proof
-> material boundary or objection -> available next action. Adapt rather than use
this as a mandatory surface formula.

**Review:** claim substantiation, offer clarity, audience fit, funnel continuity,
real CTA, measurement, brand memory, accessibility, ethics, and legal constraints.

## Informational

**Input:** reader task, source hierarchy, version/environment, definitions,
prerequisites, risks, expected result, edge cases, recovery path.

**Architecture:** answer or goal -> prerequisites -> ordered explanation/procedure
-> verification -> failure/recovery -> references or next step.

**Review:** correctness, completeness for the stated task, evidence, dependency
order, scannability, reproducibility, caveats, terminology, and freshness.

## Communication

**Input:** sender, recipient, relationship, prior state, desired state, facts,
decision/action, owner, timing, consequence, reply or escalation path.

**Architecture:** relevant state/decision -> requested or available action -> owner
and timing -> necessary context -> reply/update path.

**Review:** recipient can act, authority is accurate, tone matches consequence,
responsibility is visible, uncertainty is honest, and no blame or urgency is
manufactured.

## Progressive Readiness

| Level | Required condition |
| --- | --- |
| Contract | Purpose, final inventory, SOT, channel, and acceptance are agreed; no deliverable claim yet |
| Functional 1% | A short complete text performs the production purpose end to end without fabricated support |
| Structured draft | Every final section/function exists; evidence and examples are integrated or honestly marked unresolved |
| Reviewed draft | Purpose, truth, structure, mode, voice, grammar, and channel checks passed with findings resolved |
| Channel-ready | Links, formatting, accessibility, legal/brand constraints, CTA or resolution, and active version are verified |
| Outcome-validated | Real reader, conversion, comprehension, support, editorial, or other product evidence informed the next pass |

### Functional 1% Examples

- Literary: a 300-word scene reaches the planned turn and changes the situation;
  later passes deepen texture, subtext, and rhythm.
- Marketing: a short live product block states the qualified offer, honest proof
  boundary, price/condition if material, and working CTA; later passes add tested
  objections, variants, and channel detail.
- Informational: a concise answer lets the reader complete the main task and
  verify success; later passes add alternatives, examples, and edge cases.
- Communication: a brief message gives the decision, action owner, deadline or
  next-update time, and reply path; later passes add context and tone calibration.
